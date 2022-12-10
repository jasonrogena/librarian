## Filesystem Librarian

[![Build Status](https://github.com/jasonrogena/librarian/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jasonrogena/librarian/actions/workflows/ci.yml?query=branch%3Amain+workflow%3ACI)
[![codecov](https://codecov.io/gh/jasonrogena/librarian/branch/main/graph/badge.svg?token=O3PNGORLW8)](https://codecov.io/gh/jasonrogena/librarian)

Librarian runs pre-configured commands against a group of files that match a set of filters. The group of files is called a library. Librarian can either search for files in a library or watch for when files in the library are created or updated.

To run Librarian once, where it exits after searching through a list of configured libraries, run:

```sh
fs-librarian single-shot path/to/config.toml
```

To make Librarian continually watch for when files in configured libraries are created or updated, run:

```sh
fs-librarian watch path/to/config.toml
```

### Building

You can use the pre-built binaries on the [release page](./releases) or build Librarian on your own. To build Librarian, make sure you have Rust installed on your machine (installation instructions are [here](https://www.rust-lang.org/tools/install)) then run:

```sh
make clean build
```

The binary `target/release/fs-librarian` will be generated.

### Configuration

An example configuration file can be found [here](./tests/configs/good.toml).

#### Libraries

In the Librarian configuration file, define one or more "libraries" of files. A library is a set of files that match defined search filters. Supported search filters are:

- A required list of parent directories the file can be in
- An optional list of regexes the file's MIME type should match

For each of the defined libraries, provide a [Tera template](https://tera.netlify.app/docs/#templates) (whose syntax is based on Jinja2) of the command that should run when a file is found. The following variables are available to the template:

- `{{ file_path }}`: The path to the file that was found
- `{{ mime_type }}`: The MIME type for the file that was found. Run the `fs-librarian test mime <path to a file>` command to display the MIME types of files you are unsure about.

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

### Considerations

Consider the following when using Librarian:

- Librarian does not limit itself to files in the root of the configured filter directories. It will also consider files in sub-directories.
- The pre-configured commands will run concurrently against your libraries. In single-shot mode, a separate thread will be used for each of the configured libraries. Watch mode will use a separate thread for each file-update notification. Race conditions might occur if the same file matches the filters for more than one library or if a pre-configured command you provide isn't safe to be run more than once, concurrently, against the same file.
- Librarian relies on OS-specific MIME-type databases. Therefore, it is possible for the same file to appear to have a different MIME-type on different OSs.
- In watch mode, expect that the pre-configured command will be called more than once when a file is created or updated (once for each file-update notification emitted by the OS). Some OSs emit more than one notification (e.g. `IN_CREATE` and `IN_CLOSE_WRITE` on Linux) when a file is changed. You can avoid the pre-configured command from running more than once for every file update using the `min_command_exec_freq` option.
- Use absolute paths in your configuration files. Librarian might not behave as expected if you use relative paths.

### License

[MIT](./LICENSE)
