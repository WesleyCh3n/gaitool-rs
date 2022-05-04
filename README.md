# Analyze Gait

[![Continuous Integration](https://github.com/WesleyCh3n/analyze.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/WesleyCh3n/analyze.rs/actions/workflows/ci.yml)

This is a command line tools to analyze human's gait data, including
preprocessing, calculating gait cycle, remapping columns etc. Then
the output is valid json api to
[analyze.api](https://github.com/wesleych3n/analyze.api/).

## Getting Started

### Usage

```
USAGE:
analyze-rs <SUBCOMMAND>

OPTIONS:
-h, --help    Print help information

SUBCOMMANDS:
concat
export
filter
help      Print this message or the help of the given subcommand(s)
swrite
```

#### filter

Specify `-f` input data and `-s` save directory. To filter raw/remapped data
(including info header), then output valid data.

Example input:
```shell
analyze-rs filter -f ./file/raw/sample.csv -s file/csv
```

Example output:
```shell
{
  "FltrFile":{
    "cyDb":"db.csv",
    "cyGt":"gait.csv",
    "cyLt":"ls.csv",
    "cyRt":"rs.csv",
    "rslt":"sample.csv"
  },
  "Range":[{"End":15.965,"Start":4.37},{"End":35.755,"Start":25.375}]
}
```

#### export

Specify `-f` input data, `-s` save directory and `-r` follow by string with two
number with a space separated to select valid range in gait cycle (able to
select multiple ranges). To export each node max/min mean in valid gait cycle.

Example input:
```shell
analyze-rs export -f file/csv/sample.csv -s file/export -r "1 12" -r "25 33"
```

Example output:
```shell
{"ExportFile": "sample-result.csv"}
```

#### swrite

Specify `-f` input data, `-s` save directory and `-v` follow by a string which
is valid time range, a space separate each range and `-` in each range.
To write the valid selection time to new columns selected file.

Example input:
```shell
analyze-rs swrite -f file/raw/sample.csv -s file/export -v "4.37-15.965 18.06-22.00"
```

Example output:
```shell
{"CleanFile":"sample.csv"}
```

#### concat

Specify `-f` input data (multiple), `-s` save directory. To concatenate
multiple export result in one file.

Example input:
```shell
analyze-rs concat -f file/export/sample-result-1.csv -f file/export/sample-result-2.csv -s file/export
```

Example output:
```shell
{"ConcatFile":"concat.csv"}
```

