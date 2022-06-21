## Librarian

[![Build Status](https://github.com/jasonrogena/librarian/workflows/CI/badge.svg)](https://github.com/jasonrogena/librarian/actions?query=workflow%3ACI)[![codecov](https://codecov.io/gh/jasonrogena/librarian/branch/main/graph/badge.svg?token=O3PNGORLW8)](https://codecov.io/gh/jasonrogena/librarian)

Librarian searches for files in your filesystem and runs the configured commands against discovered files.

To run Librarian once, where it exits after searching through all the libraries, run:

```sh
librarian single-shot path/to/config.toml
```

To make Librarian continually watch for when files in the libraries change (are created or updated), run:

```sh
librarian watch path/to/config.toml
```

### Configuration

An example configuration file can be found [here](./tests/configs/good.toml).

#### Libraries

In the Librarian configuration file, define one or more "libraries" of files. A library is a set of files that match defined search filters. Supported search filters are:

- A required list of parent directories the file can be in
- An optional list of regexes the file's MIME type should match

For each of the defined libraries, provide a [Jinja2](https://jinja.palletsprojects.com/en/2.10.x/templates/) template of the command that should run when a file is found. The following variables are available to the template:

- `{{ file_path }}`: The path to the file that was found
- `{{ mime_type }}`: The MIME type for the file that was found

The following configuration snippet defines a music library which watches for files inside the Downloads and /tmp directories that have MIME types matching the `audio/.+` regex (e.g. `audio/flac` and `audio/ogg`). When an audio file is found, it is moved to the Music directory:

```toml
[libraries.music]
command = """
mv "{{ file_path }}" /home/jrogena/Music/
"""

  [libraries.music.filter]
  directories = [ "/home/jrogena/Downloads", "/tmp" ]
  mime_type_regexes = [ "audio/.+" ]
```

#### Filesystem Watching

The following configurations, related to filesystem watching, are available:

 - `min_command_exec_freq`: Optional. The minimum frequency (in seconds) between running the configured command against a file. Useful in situations where a file is updated frequently but you don't want Librarian to run against the file as frequently as it is updated.

 The following snippet is an example filesystem watching configuration:

```toml
[fs_watch]
min_command_exec_freq = 60
```

### License

[MIT](./LICENSE)
