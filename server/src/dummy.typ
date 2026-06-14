// Generates an invoice
#let invoice(
  data,
) = {
  set text(lang: "de", region: "DE")

  set page(paper: "a4", margin: (x: 20%, y: 20%, top: 20%, bottom: 20%))

  set text(number-type: "old-style")

  let invoice-date = datetime(year: data.date.year, month: data.date.month, day: data.date.day)
  let (number: invoice-nr, author, recipient) = data

  smallcaps[
    *#author.name* •
    #author.street •
    #author.zip #author.city
  ]

  v(1em)

  [
    #set par(leading: 0.40em)
    #set text(size: 1.2em)
    #recipient.name \
    #recipient.street \
    #recipient.zip
    #recipient.city
  ]

  v(4em)

  grid(
    columns: (1fr, 1fr),
    align: bottom,
    heading[
      Rechnung \##invoice-nr
    ],
    [
      #set align(right)
      #author.city, *#invoice-date.display("[day].[month].[year]")*
    ],
  )
}
