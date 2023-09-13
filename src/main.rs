use std::{
    fs::{create_dir_all, File},
    io::{Error, ErrorKind, Read, Seek, SeekFrom, Write},
    path::Path,
    str::from_utf8,
};

// It's in one file because I hate you

// Only supporting u32 because there is absolutely no fucking reason to do u64 on a GC game
fn read_word(f: &mut File, offset: u32) -> Result<u32, Error> {
    let mut buffer = vec![0; 4];

    f.seek(SeekFrom::Start(offset.into()))?;
    f.read(&mut buffer)?;

    let word = u32::from_be_bytes(buffer.try_into().unwrap());
    Ok(word)
}

// Maybe expand later for files that get chonky (u64)
fn read_bytes(f: &mut File, offset: u32, size: u32) -> Result<Vec<u8>, Error> {
    let mut buffer = vec![0; size.try_into().unwrap()];
    f.seek(SeekFrom::Start(offset.into()))?;
    f.read(&mut buffer)?;

    Ok(buffer)
}

// Why are we here, just to suffer
fn read_byte(f: &mut File, offset: u32) -> Result<u8, Error> {
    let byte = read_bytes(f, offset, 0x1)?;

    Ok(byte[0])
}

// Reads strings we know the size of, a bit shite 2BH
fn read_string(f: &mut File, offset: u32, size: u32) -> Result<String, Error> {
    let mut buffer = read_bytes(f, offset, size)?;

    match from_utf8(&mut buffer) {
        Ok(v) => Ok(v.to_string()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

fn write_file(buffer: Vec<u8>, name: String) -> Result<(), Error> {
    let mut path = "./files/".to_owned();
    create_dir_all(&path)?;

    path.push_str(&name);
    let file = Path::new(&path);

    let mut f = File::create(&file)?;
    f.write_all(&buffer)?;

    Ok(())
}

// Stores all data pertaining to boot.bin
#[derive(Debug)]
#[allow(dead_code)]
struct BootData {
    addr: u32,
    size: u64,
    id: String,
    maker_code: String,
    bootfile_addr: u32,
    fst_addr: u32,
    fst_size: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Bi2Data {
    addr: u32,
    size: u64,
}

// Yeah, I'm not touching this one with a 7 foot barge pole
#[derive(Debug)]
#[allow(dead_code)]
struct AppldrData {
    addr: u32,
    size: u64,
}

// Actually sort of important, tells us where all our shit is
fn get_boot_data(f: &mut File) -> Result<BootData, Error> {
    Ok(BootData {
        addr: 0x0,
        size: 0x440,
        id: read_string(f, 0x0, 0x4)?,
        maker_code: read_string(f, 0x4, 0x2)?,
        bootfile_addr: read_word(f, 0x420)?,
        fst_addr: read_word(f, 0x424)?,
        fst_size: read_word(f, 0x428)? as u64,
    })
}

fn get_bi2_data() -> Bi2Data {
    Bi2Data {
        addr: 0x440,
        size: 0x2000,
    }
}

fn get_appldr_data(f: &mut File) -> Result<AppldrData, Error> {
    let appldr_size = read_word(f, 0x2454)? + read_word(f, 0x2458)?;

    Ok(AppldrData {
        addr: 0x2440,
        size: appldr_size as u64,
    })
}

// Fucking send that shit baby
fn read_til_dead(f: &mut File, offset: u32) -> Result<String, Error> {
    let mut string = "".to_string();
    f.seek(SeekFrom::Start(offset as u64))?;

    loop {
        let mut buffer = vec![0; 1];
        f.read(&mut buffer)?;

        match buffer[0] {
            0x0 => break,
            _ => {
                let cha = from_utf8(&buffer).unwrap();
                string.push_str(cha);
            }
        }
    }

    Ok(string)
}

// Because it's apparently cool as fuck to store things as a u24
fn read_u24_as_u32(f: &mut File, offset: u32) -> Result<u32, Error> {
    let mut buf = [0; 4];
    f.seek(SeekFrom::Start(offset as u64))?;
    f.read_exact(&mut buf[1..])?;
    Ok(u32::from_be_bytes(buf))
}

#[derive(Debug)]
#[allow(dead_code)]
struct FstDir {
    name: String,
    start: u32,
    end: u32,
}

fn build_fst_dir(
    f: &mut File,
    base_offset: u32,
    dir_offset: u32,
    str_table: u32,
) -> Result<FstDir, Error> {
    let name_offset = read_u24_as_u32(f, base_offset + dir_offset + 0x1)?;
    let mut name = read_til_dead(f, str_table + name_offset)?;
    let dir_end = read_word(f, base_offset + dir_offset + 0x8)?;

    // If it's the root directory, just have an empty string
    if name_offset == 0x0 {
        name = String::new();
    }

    Ok(FstDir {
        name: name,
        start: dir_offset / 0xC,
        end: dir_end,
    })
}

#[derive(Debug)]
#[allow(dead_code)]
struct FstFile {
    name: String,
    path: String,
    start: u32,
    end: u32,
}

fn build_fst_file(
    f: &mut File,
    base_offset: u32,
    file_offset: u32,
    str_table: u32,
    dir_path: String,
) -> Result<FstFile, Error> {
    let name_offset = read_u24_as_u32(f, base_offset + file_offset + 0x1)?;
    let file_start = read_word(f, base_offset + file_offset + 0x4)?;
    let file_end = read_word(f, base_offset + file_offset + 0x8)?;
    let name = read_til_dead(f, str_table + name_offset)?;

    Ok(FstFile {
        name: name,
        path: dir_path,
        start: file_start,
        end: file_start + file_end,
    })
}

fn get_dir_path(file_id: u32, dir_vec: &Vec<FstDir>) -> String {
    let mut string = "".to_owned();

    for dir in dir_vec {
        if (file_id >= dir.start) & (file_id < dir.end) {
            string.push_str(&(dir.name.to_owned() + "/"));
        }
    }

    return String::from(string);
}

fn read_fst(f: &mut File, offset: u32) -> Result<Vec<FstFile>, Error> {
    let num_entries = read_word(f, offset + 0x8)?;
    let str_table = offset + (num_entries * 0xC);
    let mut dir_vec: Vec<FstDir> = vec![];
    let mut file_vec: Vec<FstFile> = vec![];

    for i in 0..num_entries {
        let file_offset = 0xC * i;

        match read_byte(f, offset + file_offset) {
            Ok(0x1) => {
                let fstdir = build_fst_dir(f, offset, file_offset, str_table)?;
                dir_vec.push(fstdir);
            }
            Ok(0x0) => {
                let fstfile =
                    build_fst_file(f, offset, file_offset, str_table, get_dir_path(i, &dir_vec))?;
                file_vec.push(fstfile);
            }
            _ => println!("Your file is beyond fucked lmfao"),
        }
    }

    Ok(file_vec)
}

fn write_fst_file(f: &mut File, file: FstFile) {
    f.seek(SeekFrom::Start(file.start as u64)).unwrap();

    let mut buffer = vec![0; (file.end - file.start) as usize];
    f.read(&mut buffer).unwrap();

    if file.path != "/" {
        create_dir_all(format!("./files{}", &file.path)).unwrap();
    }
    write_file(buffer, format!("{}{}", file.path, file.name)).unwrap();
}

fn write_fst_files(f: &mut File, files: Vec<FstFile>) {
    for file in files {
        write_fst_file(f, file);
    }
}

fn main() -> Result<(), Error> {
    let mut f = File::open("./iso/gladius.iso")?;

    let boot = get_boot_data(&mut f)?;
    let bi2 = get_bi2_data();
    let appldr = get_appldr_data(&mut f)?;

    let bootbuf = read_bytes(&mut f, boot.addr, boot.size as u32)?;
    write_file(bootbuf, String::from("boot.bin"))?;

    let bi2buf = read_bytes(&mut f, bi2.addr, bi2.size as u32)?;
    write_file(bi2buf, String::from("bi2.bin"))?;

    let appldrbuf = read_bytes(&mut f, appldr.addr, appldr.size as u32)?;
    write_file(appldrbuf, String::from("appldr.bin"))?;

    let fstbuf = read_bytes(&mut f, boot.fst_addr, boot.fst_size as u32)?;
    write_file(fstbuf, String::from("fst.bin"))?;

    let files = read_fst(&mut f, boot.fst_addr)?;

    // Zug zug work done
    write_fst_files(&mut f, files);

    Ok(())
}
