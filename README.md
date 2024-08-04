<!--
SPDX-FileCopyrightText: 2024 Benedikt Bastin

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# assetinfo

## About

assetinfo is a tool to watch for versions of assets and their end-of-life date.

## Installation

You can install assetinfo using Cargo.

```bash
cargo install assetinfo
```

## Usage

assetinfo requires a config file as well as a database of supported programs, which can be updated using assetinfo itself. The minimal required configuration states the path of the database and a URL from which updates can be fetched. It can be seen below. By default, assetinfo looks for the configuration file in the current working directory under the name **assetinfo-config.toml**.

```toml
[database]
path = "./db"
update_url= "https://db.assetinfo.de/latest.tar.zstd"
```

> [!CAUTION]
> At the time of writing, there is **no signing and validation** process in place.
> The files do contain commands that will be executed
> which could **delete or leak data or otherwise harm your system**.
> Be sure to manually validate all program files after download.

In order to update your database, run `assetinfo update`. This will download an update your local database.

To list the programs in your database, run `assetinfo list`. You can find an example output below.

```
 Supported programs
-----------------------------+-------------------------------+--------+--------
 Program Name                | Program ID                    | Binary | Docker
 Ansible Community           | com.ansible.ansible-community | true   | false
 Ansible Core                | com.ansible.ansible-core      | true   | false
 Apache HTTP Server          | org.apache.httpd              | true   | false
 Debian                      | org.debian                    | true   | false
 Docker Engine               | com.docker                    | true   | false
 Firefox                     | org.mozilla.firefox           | true   | false
 Grocy                       | org.grocy                     | false  | true
 MariaDB                     | org.mariadb                   | true   | false
 Mattermost                  | com.mattermost                | true   | true
 PHP: Hypertext Preprocessor | net.php                       | true   | false
 PostgreSQL                  | org.postgresql                | true   | true
 Ubuntu                      | com.ubuntu                    | true   | false
 linux                       | org.kernel.linux              | true   | false
 nginx                       | org.nginx                     | true   | true
 python                      | org.python                    | true   | false
 systemd                     | org.freedesktop.systemd       | true   | false
```

To get information on one program, use `assetinfo info <program>`. Program can be either the name or the program ID.

```
assetinfo info org.python
python (Binary) found in Version 3.12.4
Version 3.12 will be supported for 1549 days (2028-10-31)
python (Binary) found in Version 3.12.4
Version 3.12 will be supported for 1549 days (2028-10-31)
```

To check for all supported programs, run `assetinfo info-all`.

## Contributing

Right now, I sadly do not have the time to accept contributions.
But I greatly appreciate feedback and hope this might change in the future.

## Authors and Acknowledgment

This README is inspired by [makeareadme.com][makeareadme].

## License

The code of this project is licensed under the **AGPL 3.0 or later**. The
documentation is licensed under **CC-BY-SA 4.0**. Some small configuration
files (which probably do not reach the [**threshold of originality**][
threshold]) are given a license with no conditions under **CC0 1.0**.

See [LICENSES](LICENSES) for more information.


[makeareadme]: https://www.makeareadme.com/
[threshold]: https://en.wikipedia.org/wiki/Threshold_of_originality
