pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub trait MessageHeader {
	
}

pub struct HeaderSend {
	version: u32,
	payload: u32,
	mtype:   u32,
	flags:   u32,
	offset:  u32,
}

pub struct HeaderReceive {
	version: u32,
	payload: u32,
	ret:     u32,
	flags:   u32,
	offset:  u32,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
