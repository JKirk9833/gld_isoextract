# The Gladius Iso

This will be more of a backup as the websites that I check for this information
have a bad tendency of disappearing, taking the content with them.

This document should contain all the information needed to construct or
deconstruct the Gladius.iso (and other .iso files as a side effect).

The information within is not relevant outside of the packing and unpacking of
the iso to get to the files, as Xexu and JimB16 have both solved this, the
information below is purely for reference.

## How is the .iso constructed?

> The general structure can be split into 5 different sections, Files / Unknown
> don't count really as they're just generic data.

| File         | Description                                                   |
| ------------ | ------------------------------------------------------------- |
| boot.bin     | Contains disk header information                              |
| bi2.bin      | Contains region disk information                              |
| appldr.bin   | A small program layer between GC bootrom and main application |
| bootfile.dol | The executable (where our game code lives)                    |
| fst.bin      | Contains the location of generic game assets                  |
| Files        | The actual files for the game, refer to fst.bin for locations |
| Unknown      | No idea what these are, needs looking into eventually         |

### boot.bin

> This section will contain all the knowledge I currently have on boot.bin

Location: 0x0 - 0x440

Information from: [HERE](https://wiki.gbatemp.net/wiki/NKit/Discs)

Specifically the GameCube Disc / Wii Raw Partition Data section.

| Item No. | Offset | Length | Name                  |
| -------- | ------ | ------ | --------------------- |
| 1        | 0x0    | 0x4    | ID                    |
| 2        | 0x4    | 0x2    | Maker Code            |
| 3        | 0x5    | 0x1    | Disc No               |
| 4        | 0x6    | 0x1    | Disc Version          |
| 5        | 0x20   | 0x40   | Title                 |
| 6        | 0x420  | 0x4    | bootfile.dol location |
| 7        | 0x424  | 0x4    | fst.bin location      |
| 8        | 0x428  | 0x4    | fst.bin size          |
| 9        | 0x42C  | 0x4    | Max fst.bin size      |

### bi2.bin

> Contains region information and that's about it, we can mostly ignore this

Location: 0x440 - 0x2000

### appldr.bin

> A small program that runs before the game as a staging thing by the looks.
> Please note that this maybe wrong, but I can't see any other reason for it.

Location: 0x2440 - (0x2440 + app size)

We need to find the app size to be able to determine the size of the appldr.bin.
To do this we can add the u32's contained within offsets 0x14 and 0x18 plus 32.
The reason for the 32 is that the size does not include the header itself, the
header being 32 bytes long.

### bootfile.dol

> Contains our game code, this includes everything from bleed damage to model
> loading and input handling - the problem being that it is compiled :(
> If you find the native symbol data for GameCube that would be great.

Location: Pointed to by boot.bin offset 0x420.

All extra information about the .dol, such as information on how to modify it to
behave the way you want it to will be within its own section as it can get quite
complicated and I don't want to overload this section with it.

This file only has two sections, the header and the content. The content itself
is split into 7 text sections and 11 data sections. If you're looking for code,
the text sections are where it's at; anything else and you will want to search
the data sections.

The header for the file will always be 0x100 bytes and contains the following:

| Offset | Size     | Description                                          |
| ------ | -------- | ---------------------------------------------------- |
| 0x000  | 0x4 × 18 | Pointer to section data start location               |
| 0x048  | 0x4 × 18 | Where the data should be copied as a virtual address |
| 0x090  | 0x4 × 18 | The size of the data in bytes                        |
| 0x0d8  | 0x4      | Start of the bss / block starting symbol             |
| 0x0dc  | 0x4      | Size of the bss                                      |
| 0x0e0  | 0x4      | Pointer to the main method of the .dol               |
| 0x0e4  | 0x1c     | Padding.                                             |
| 0x100  | 0x0      | The end of the header                                |

Sections 0x000 / 0x048 / 0x090 can all be read as 3 different arrays containing
one type of data in each. The data a whole can be read using an index.

Something to note, you don't really want to edit this file directly - instead
you may want to use the Gekko codes system for Dolphin to inject your jump
instructions and alter the assembly from there. Keep in mind the registers that
you overwrite; information about this will be stored in the Gekko section.

### fst.bin

> Contains the location of game assets / anything that shows up visually ingame

Location: Pointed to by boot.bin 0x424, size pointed to by 0x428.

The FST is split into a few different segments:

| Segment      | Description                                                   |
| ------------ | ------------------------------------------------------------- |
| File Entries | This segment is filled with 0xC size file entries (see below) |
| File Names   | This segment simply contains strings for file names           |
| Paddings     | This is just 0x0 byte alignment to a multiple of 4 bytes      |

A file entry consists of 0xC / 12 bytes:

| Name        | Offset | Length | Description                                |
| ----------- | ------ | ------ | ------------------------------------------ |
| File/Folder | 0x0    | 0x1    | 0x1 is a folder, 0x0 is a file             |
| File Name   | 0x1    | 0x3    | Refers to the name offset within "names"   |
| File Offset | 0x4    | 0x4    | Needs a fair bit of explanation, see below |
| File Size   | 0x8    | 0x4    | Size of the file or # of entries in folder |

**File Offset**

This section is different, depending on whether the entry is a file or a folder.
If this section is flagged with a 0x1 (folder), then it refers to entry number
of the parent directory, the first file entry will ALWAYS be "ROOT", therefore
it will have an entry number of 0x0.

If this section is flagged with a 0x0 (file), then it refers to an offset i.e.
the location of the file that the section refers to.

Examples of file entries are below:

| File/Folder | File Name | File Offset | File Size   | Description         |
| ----------- | --------- | ----------- | ----------- | ------------------- |
| 01          | 00 00 00  | 00 00 00 00 | 00 00 00 1C | Root directory      |
| 00          | 00 00 00  | 00 34 00 00 | 27 65 E8 00 | Top Level File      |
| 01          | 00 00 0A  | 00 00 00 00 | 00 00 00 19 | Top Level Directory |

### Files

> The data that the fst.bin files refers to, this is the actual file data
> within the Gladius.iso, this will contain audio.bec and gladius.bec alongside
> a bunch of cinematics (which can and should be replaced with TROGDOR)

There will be a section within "Modding/" that will cover what the file types
are and how to edit them; this information does not belong here so go there if
you need to know more.

## Further Reading

> If you want to learn more, there's a lot of good relevant info here

- https://wiki.gbatemp.net/wiki/NKit/Discs
- https://gbatemp.net/threads/fst-bin-file-structure.78241/
- http://hitmen.c02.at/files/yagcd/yagcd/chap13.html
- https://www.gc-forever.com/wiki/index.php?title=Apploader
- https://wiki.tockdom.com/wiki/DOL_(File_Format)
