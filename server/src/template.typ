#import "@preview/classy-german-invoice:0.3.2": invoice

#let data = json("inputs")

#show: invoice(
  // Invoice number
  data.number,
  // Invoice date
  datetime(year: data.date.year, month: data.date.month, day: data.date.day),
  // Items
  data.items,
  // Author
  data.author,
  // Recipient
  data.recipient,
  // Bank account
  data.bank_account,
  // Umsatzsteuersatz (VAT)
  vat: data.vat,
  kleinunternehmer: data.kleinunternehmer,
)
