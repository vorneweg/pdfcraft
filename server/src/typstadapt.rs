use std::{collections::HashMap, env::current_dir, fs, path::PathBuf};

use typst::{
    Library, LibraryExt, World as TypstWorld,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::{
    download::{Downloader, ProgressSink},
    fonts::{FontSearcher, FontSlot, Fonts},
    package::PackageStorage,
};

pub struct World {
    lib: LazyHash<Library>,
    font_slots: Vec<FontSlot>,
    font_book: LazyHash<FontBook>,
    main: Source,
    storage: PackageStorage,
}

impl World {
    pub fn init(source: impl Into<String>) -> anyhow::Result<Self> {
        let cache_path = current_dir().unwrap().join(".typst-cache");
        fs::create_dir_all(&cache_path)?;
        let downloader = Downloader::new("pdfcraft-dev");
        let storage = PackageStorage::new(Some(cache_path), None, downloader);

        let Fonts { book, fonts } = FontSearcher::new().include_system_fonts(false).search();

        let main_id = FileId::new_fake(VirtualPath::new("main"));
        let main = Source::new(main_id, source.into());

        Ok(Self {
            lib: LazyHash::new(Library::builder().build()),
            font_slots: fonts,
            font_book: LazyHash::new(book),
            main,
            storage,
        })
    }

    fn find_file(&self, id: &FileId) -> Result<PathBuf, FileError> {
        if let Some(pkg) = id.package() {
            let pkg_path = self.storage.prepare_package(pkg, &mut ProgressSink)?;
            return Ok(pkg_path.join(id.vpath().as_rootless_path()));
        }
        // don't support local files
        Err(FileError::AccessDenied)
    }
}

impl TypstWorld for World {
    fn library(&self) -> &LazyHash<Library> {
        &self.lib
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.font_book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        let bytes = self.file(id)?;
        let text = bytes.to_str()?.to_string();
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main.id() {
            return Ok(Bytes::from_string(self.main.text().to_owned()));
        }
        let path = self.find_file(&id)?;
        let bytes = fs::read(&path).map_err(|e| {
            use std::io::ErrorKind::*;
            match e.kind() {
                NotFound => FileError::NotFound(path),
                IsADirectory => FileError::IsDirectory,
                e => FileError::Other(Some(e.to_string().into())),
            }
        })?;
        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.font_slots.get(index).and_then(|s| s.get())
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

pub struct WorldWithDataOverlay<'a> {
    world: &'a World,
    data: HashMap<VirtualPath, Bytes>,
}

impl<'a> WorldWithDataOverlay<'a> {
    pub fn new(world: &'a World, data: impl IntoIterator<Item = (VirtualPath, Bytes)>) -> Self {
        Self {
            world,
            data: data.into_iter().collect(),
        }
    }
}

impl<'a> TypstWorld for WorldWithDataOverlay<'a> {
    fn library(&self) -> &LazyHash<Library> {
        self.world.library()
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.world.book()
    }

    fn main(&self) -> FileId {
        self.world.main()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id.package().is_none() {
            if let Some(data) = self.data.get(id.vpath()) {
                let text = data.to_str()?.to_string();
                return Ok(Source::new(id, text));
            }
        }
        self.world.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id.package().is_none() {
            if let Some(data) = self.data.get(id.vpath()) {
                return Ok(data.clone());
            }
        }
        self.world.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.world.font(index)
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        self.world.today(offset)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::World;
    use crate::model::EXAMPLE_DATA;

    use serde::Serialize;
    use typst::{
        World as _,
        comemo::Track,
        engine::{Engine, Route, Sink, Traced},
        foundations::{StyleChain, Target, TargetElem, Value},
        introspection::Introspector,
    };
    use typst_pdf::{PdfOptions, pdf};

    const DUMMY_TEMPLATE: &str = include_str!("dummy.typ");

    fn into_value_serde(from: impl Serialize) -> Value {
        let json_value = serde_json::to_value(from).unwrap();
        serde_json::from_value(json_value).unwrap()
    }

    #[test]
    fn typst_minimal() {
        // parse
        let world = World::init(DUMMY_TEMPLATE).unwrap();
        let main = world.main.clone();
        let world = (&world as &dyn typst::World).track();

        let mut sink = Sink::new();
        let traced = Traced::default();
        let module = typst_eval::eval(
            &typst::ROUTINES,
            world,
            traced.track(),
            sink.track_mut(),
            Route::default().track(),
            &main,
        )
        .unwrap();

        let Value::Func(invoice) = module.field("invoice", ()).unwrap() else {
            panic!("invalid field type");
        };

        let introspector = Introspector::default();
        let constraint = typst::comemo::Constraint::new();
        let mut engine = Engine {
            world,
            introspector: introspector.track_with(&constraint),
            traced: traced.track(),
            sink: sink.track_mut(),
            route: Route::default(),
            routines: &typst::ROUTINES,
        };

        let context = typst::foundations::Context::default();
        let args = [into_value_serde(&*EXAMPLE_DATA)];
        let Value::Content(content) = invoice.call(&mut engine, context.track(), args).unwrap()
        else {
            panic!("return value was not content")
        };

        let library = world.library();
        let base = StyleChain::new(&library.styles);
        let target = TargetElem::target.set(Target::Paged).wrap();
        let styles = base.chain(&target);

        let doc = typst_layout::layout_document(&mut engine, &content, styles).unwrap();
        let out = pdf(&doc, &PdfOptions::default()).unwrap();
        fs::write("dummy.pdf", out).unwrap();
    }
}
