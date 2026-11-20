mod device;

fn main() -> i32 {
    var handle = device.make();
    device.append(handle);
    let name = device.read(handle);
    let size = name.len();
    device.close(handle);
    return (size + device.released()) as i32;
}
