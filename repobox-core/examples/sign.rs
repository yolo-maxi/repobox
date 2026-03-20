use std::io::Read;

fn main() {
    let address = std::env::args().nth(1).expect("Usage: sign <address>");
    let home = std::env::var("HOME").unwrap();

    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).unwrap();

    let sig = repobox::signing::sign(std::path::Path::new(&home), &address, &data)
        .expect("signing failed");
    print!("{}", hex::encode(&sig));
}
