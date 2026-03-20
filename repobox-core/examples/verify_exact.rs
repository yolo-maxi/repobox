use repobox::signing::recover_address;

fn main() {
    let msg = "0x1FC519618b8C8902e8FB52f1A23A8961675DD0e9/private-test:1774030759";
    let sig_hex = "d9353144746fee0e7c3112213c493b63b865b965a6462fd5283bbc323ff17374064d959af5731d66efcabd2aa01a155987849a88e5098f873d3195a1aa48c1c700";
    
    let sig_bytes = hex::decode(sig_hex).expect("decode failed");
    println!("Message: {}", msg);
    println!("Sig bytes len: {}", sig_bytes.len());
    
    match recover_address(msg.as_bytes(), &sig_bytes) {
        Ok(addr) => println!("Recovered: {}", addr),
        Err(e) => println!("Error: {:?}", e),
    }
}
