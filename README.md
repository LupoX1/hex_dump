cargo build --release

executable:
* hex_dump

parameters:
* -i path/to/input_file.ext (mandatory)
* -c columns [8,16,32,64] (optional)

output:
* path/to/input_file.dump