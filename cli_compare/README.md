### CLI Comparer

Консольное приложение, использующее функциональность парсеров из lib-крейта.

CLI Comparer должен читать данные о транзакциях из двух файлов и сравнивать их. Входные файлы могут быть в любых форматах, которые поддерживаются парсерами из lib-крейта. В случае несовпадения, утилита должна сообщать какая транзакция не совпала.

Пример запуска утилиты:
```terminaloutput
ypbank_compare --file1 records_example.bin --format1 binary --file2 records_example.csv --format2 csv
# Output: The transaction records in 'records_example.bin' and 'records_example.csv' are identical.
```

### Example
```terminaloutput
cargo run --bin cli_compare -- --file1 ./example_files/records_example.txt --format1 txt --file2 ./example_files/records_example.txt --format2 txt
```