### CLI Converter

Консольное приложение, использующее функциональность парсеров из первого крейта.

Парсеры крейта «парсер» принимают в качестве входа типы, удовлетворяющие трейту Read. CLI-приложение должно читать данные из [файла](https://doc.rust-lang.org/std/io/trait.Read.html#impl-Read-for-%26File) и выводить результат в `stdout`. Для библиотеки ничего не меняется — она работает с абстракциями, определёнными в стандартной библиотеке, реализованными для типа `File` и доступными в любом Rust-приложении.

За счёт статического полиморфизма (мономорфизации) в итоговый бинарник попадут только те реализации трейтов, которые реально используются.

Пример запуска утилиты:
```terminaloutput
ypbank_converter \
  --input <input_file> \
  --input-format <format> \
  --output-format <format> \
  > output_file.txt
```


### Example
```terminaloutput
cargo run --bin cli_converter -- --input ./example_files/records_example.txt --input-format=txt --output-format=csv > ./tmp/some.csv
```