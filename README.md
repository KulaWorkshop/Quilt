<h4 align="center">
    <a href="https://github.com/KulaWorkshop/Quilt/releases">Releases</a> |
    <a href="https://docs.kula.quest/tools/quilt">Documentation</a>
</h4>

<div align="center">

# Quilt

[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
![Version](https://img.shields.io/badge/Version-1.0.0-orange)

![Quilt Screenshot](./.github/screenshot.png)

</div>

**Quilt** is a command-line tool that can extract and create archive files from Kula Quest, such as **.PAK** and **.KUB** files.
It can also decompress and compress files using the [LZRW3-A](http://www.ross.net/compression/lzrw3a.html) algorithm used in the earliest release of the game.

## Usage

To extract an existing archive file, use the **unpack** command followed by the input archive and a directory to extract the contents to.
The following example command will extract the files inside of `HIRO.PAK` into a folder called `levels`:

```bash
$ quilt unpack HIRO.PAK levels
```

Optionally, you can specify the `-f` flag when unpacking an archive, which will generate a text file containing the files extracted from the archive.
This is useful if you would like to rebuild an archive using the structure, without having to specify every file individually.

To create an archive file, use the **pack** command followed by the output archive file to create and a list of files to use:

```bash
$ quilt pack LEVELS.PAK LEVEL_1 LEVEL_2 LEVEL_3
```

If you would like to specify a text file containing the names of files to add to the archive, you can use the **@** parameter.
The following example command will create an archive using the filenames inside of the `HIRO.PAK.txt` file:

```bash
$ quilt pack HIRO.PAK @HIRO.PAK.txt
```

By default, a **.PAK** file will be created.
In order to create a **.KUB** file, specify the the `-k` flag in the pack arguments.

### Alpha Compression

In the first demo release of the game, the **.TGI** and **.GGI** files are fully compressed using an older algorithm, which Quilt can handle using the **compress** and **uncompress** commands.

Here are two examples:

```bash
$ quilt compress KULA.TGI.uncompressed KULA.TGI
$ quilt decompress KULA.TGI KULA.TGI.uncompressed
```

## Credits

Developed by **[SaturnKai](https://saturnkai.dev/)**. Additionally, this tool also uses the **LZRW3-A** compression algorithm written by **[Ross N. Williams](http://www.ross.net/compression/)**.

## Changelog

**Version 1.0.0 (2025-09-14)**

-   Initial release.
