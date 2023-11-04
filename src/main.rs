use rand::rngs::ThreadRng;
use rand::Rng;
use std::net::UdpSocket;

struct Header {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16,
}

impl Header {
    fn new(
        id: u16,
        flags: u16,
        num_questions: u16,
        num_answers: u16,
        num_authorities: u16,
        num_additionals: u16,
    ) -> Header {
        Header {
            id,
            flags,
            num_questions,
            num_answers,
            num_authorities,
            num_additionals,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            self.id.to_be_bytes()[0],
            self.id.to_be_bytes()[1],
            self.flags.to_be_bytes()[0],
            self.flags.to_be_bytes()[1],
            self.num_questions.to_be_bytes()[0],
            self.num_questions.to_be_bytes()[1],
            self.num_answers.to_be_bytes()[0],
            self.num_answers.to_be_bytes()[1],
            self.num_authorities.to_be_bytes()[0],
            self.num_authorities.to_be_bytes()[1],
            self.num_additionals.to_be_bytes()[0],
            self.num_additionals.to_be_bytes()[1],
        ]
        .iter()
        .cloned()
        .collect()
    }
}

struct Question {
    name: Vec<u8>,
    type_: u16,
    class: u16,
}

impl Question {
    fn new(name: Vec<u8>, type_: u16, class: u16) -> Question {
        Question { name, type_, class }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut full_bytes = self.name.clone();

        let metadata: Vec<u8> = [
            self.type_.to_be_bytes()[0],
            self.type_.to_be_bytes()[1],
            self.class.to_be_bytes()[0],
            self.class.to_be_bytes()[1],
        ]
        .iter()
        .cloned()
        .collect();

        full_bytes.extend(metadata);

        full_bytes
    }
}

struct Record {
    name: Vec<u8>,
    type_: u16,
    class: u16,
    ttl: u16,
    data: Vec<u8>,
}

fn encode_dns_name(domain_name: String) -> Option<Vec<u8>> {
    let mut bytestring = Vec::<u8>::new();

    for segment in domain_name.split('.') {
        bytestring.push(segment.len() as u8); // TODO: better handling of len > 255
        for c in segment.chars() {
            if c.is_ascii() {
                bytestring.push(c as u8);
            } else {
                return None;
            }
        }
    }

    bytestring.push(0_u8);

    Some(bytestring)
}

fn build_query(domain_name: String, record_type: u16, mut rng: ThreadRng) -> Option<Vec<u8>> {
    let name = encode_dns_name(domain_name);
    match name {
        None => {
            return None;
        }

        Some(name_bytes) => {
            let question = Question::new(name_bytes, record_type, 1);
            // 1 = CLASS_IN = 1

            let id = rng.gen::<u16>();
            // flag indicating that recursion is desired
            let rd: u16 = 0b0000000100000000;

            let header = Header::new(id, rd, 1, 0, 0, 0);

            let mut output = header.to_bytes();
            output.extend(question.to_bytes());

            Some(output)
        }
    }
}

fn main() -> Result<(), &'static str> {
    let rng = rand::thread_rng();

    let query = match build_query(String::from("www.example.com"), 1, rng) {
        Some(q) => q,
        None => {
            return Err("invalid query");
        }
    };

    println!("sending:\n{:02x?}", query);

    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
    socket.connect(("8.8.8.8", 53)).unwrap(); //.expect("could not connect");
    socket.send(query.as_slice()).expect("couldn't send data");

    let mut buf = [0; 1024];
    let num_bytes = socket.recv(&mut buf).expect("didn't recieve data");
    let filled_buf = &mut buf[..num_bytes];

    println!("recieved:\n{:02x?}", filled_buf);

    Ok(())
}
