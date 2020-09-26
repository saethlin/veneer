use veneer::{
    fs::File,
    io::{Read, Write},
    print,
};

fn main() {
    let mut file = File::create(b"/tmp/test.foo\0").unwrap();
    file.write(&b"test contents\n"[..]).unwrap();

    let mut contents = [0; 64];
    let mut file = File::open(b"/tmp/test.foo\0").unwrap();
    let bytes_read = file.read(&mut contents).unwrap();

    print(&contents[..bytes_read]);
}
