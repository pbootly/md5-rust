/*
    Required for step 3 - Initialize the md5 buffer
*/
#[derive(Clone)]
struct Md5Buffer {
    a: u32,
    b: u32,
    c: u32,
    d: u32
}
/*
  Auxiliary functions take three 32 bit words and produce one out, to pass these to functions
  we make use of and AuxInput struct
*/
#[derive(Clone)]
struct AuxInput {
    x: u32,
    y: u32,
    z: u32
}

fn main() {
    /*
        Confirm we have an argument to use as 'input' - this is not robust
        and use of clap or similar would make sense.
        Use unwrap_or on the first argument passed and assume if nothing is
        passed we are testing an empty string.
    */
    
    let input = std::env::args().nth(1).unwrap_or("".to_string());
    let input_bytes = input.as_bytes();
    // With our input as a byte array its time to start step_one to get our padding
    let mut bit_vector = step_one(input_bytes);
    // Parse padded bit vector to ensure length is a multiple of 512
    bit_vector = step_two(bit_vector, input);
    // Initialize an Md5Buffer with defined words
    let buffer = step_three();
    // Digest it
    let digested_buffer = StepFour::step_four(buffer, bit_vector);
    // Take digest and pass it to be readable
    println!("{}", step_five(digested_buffer));
}

/*
    Function to use doubling method to convert binary to decimal. Optimization here is not
    my aim and I find it reads clearer than positional notation or using fmt!
*/
fn bin_to_dec(binary: &[u8]) -> isize {

    // Convert to string as we're making use of str_radix
    let mut bin_str = "".to_string();
    // LE convert
    let le_binary: Vec<&u8> = binary.into_iter().rev().collect();
    // Append bits to string
    for bit in le_binary {
        bin_str += &bit.to_string();
    }
    // Using str_radix make our decimal number
    let decimal = isize::from_str_radix(&bin_str.to_string(), 2).unwrap();
    decimal
}

/* 
    Function to use repeated divide by 2 to get binary from an integer
    we could instead use format!{"{:b}"} but this was a fun distraction
    from the upcoming shuffles
*/ 
fn dec_to_bin(dec: u8, e: &str, len: u8) -> Vec<u8> {
    let mut x = dec;
    let mut bv = Vec::new();
    while x > 0 {
        let b = x % 2;
        x = x / 2;
        bv.push(b);
    }

    /*
        Given an endianess return the bits in the appropriate order. This could probably
        just be a function to swap
    */
    match e {
        "little" => {
            while bv.len() < len as usize{
                bv.insert(bv.len(), 0);
            }
            bv
        },
        "big" => {
            let mut bv_copy: Vec<u8> = bv.into_iter().rev().collect();
            /* 
                Padding to the left for BE - we should specify if we want padding left or right
                Whilst the desired length is greater than the length of our bit vector,
                pad with 0's.
                [01100001]
                instead of
                [1100001]

                Because we return le with a reverse padding needs to be added after the rev if we want to
                pad left
            */
                while bv_copy.len() < len as usize{
                    bv_copy.insert(0, 0);
                }
            bv_copy
        }
        _ => panic!("Something went really wrong - no endianness received")
    }
}

fn step_one(input: &[u8]) -> Vec<u8> {

    // 3.1 Step 1. Append Padding Bits - https://tools.ietf.org/html/rfc1321#section-3.1

    // Take byte slice array &[u8] from input and put into a mutable Vec<u8> to do manipulate
    let byte_vec = input.to_vec();
    // Create a new u8 vector which will be a bit array we can work on according to rfc
    let mut bit_vec = Vec::new();

    /*
        Append to our bit vector a binary representation of our byte slices. )
    */
    for byte_slice in byte_vec {
        bit_vec.append(&mut dec_to_bin(byte_slice, "little", 8));
    }

    /*  
        Append a single bit (or in our case a u8 1 to be converted)
        from section 3.1 (step 1. Append padding bits):
        "Padding is performed as follows: a single "1" bit is appended to the message"
    */ 
    bit_vec.append(&mut dec_to_bin(1, "big", 8));

    /*
        Now we have our bit vector we need to pad with 0's until the length of the message
        becomes congruent to 448 modulo 512. For you and me that's the same shape and size or
        equal to. We're going to have a 512 bit message and later on (step two) need 64 bits
        so put simply the length of our bits (how many we have) and keep putting 0's on the end
        until the length mod 512 is 448 giving us our new message block.
    */
    while bit_vec.len() % 512 != 448 {
        bit_vec.push(0);
    }

    bit_vec
}

fn step_two(mut bit_vec: Vec<u8>, input: String) -> Vec<u8> {

    // 3.2 Step 2. Append Length - https://tools.ietf.org/html/rfc1321#section-3.2

    // Get the input length (character bytes) as bits by byte -> bit (*8) as a u32
    let length = input.chars().count() * 8;
    // Get the length of bytes as a slice of u8s to then convert to binary
    let length_bytes = length.to_le_bytes();
    /* 
        for each byte in the length of our input (number of bytes) append to bit_vec
        as little endian:
        "A 64-bit representation of b (the length of the message before the
        padding bits were added) is appended to the result of the previous
        step."
    */
    for byte in length_bytes.iter() {
        bit_vec.append(&mut dec_to_bin(*byte, "little", 8));
    }

    /*
        "At this point the resulting message (after padding with bits and with
        b) has a length that is an exact multiple of 512 bits."
    */

    bit_vec
}

fn step_three() -> Md5Buffer {

    // 3.3 Step 3. Initialize MD Buffer - https://tools.ietf.org/html/rfc1321#section-3.3

    // Create a new Md5Buffer with the defined four words used to compute the digest
    let buffer = Md5Buffer {
        a: 0x67452301,
        b: 0xEFCDAB89,
        c: 0x98BADCFE,
        d: 0x10325476
    };

    /*
        We could have done this without a function frankly, but keeping with a step per function
        it felt worth having one specifically here.
    */

    buffer
}

/*
    Step 4 has a lot going on here, so we define several functions that make the step 
*/
struct StepFour{}

impl StepFour {
    // 3.4 Step 4. Process Message in 16 word Blocks - https://tools.ietf.org/html/rfc1321#section-3.4

    fn step_four(mut buffer: Md5Buffer, bit_vector: Vec<u8>) -> Md5Buffer {
        /* 
            Work now takes place on buffer, we'll need the original buffer values
            which is defined in the rfc as:
            "Save A as AA, B as BB, C as CC, D as DD"
        */
        let mut buf_clone = buffer.clone();

        /*
            Step 4 steps appear below here. We've implemented the numeroud functions under 
            impl StepFour {} so that in main() we maintain consistency with what rfc step
            we are executing
        */

        // Get the total number of 32 bit words to process as n, where n is a multiple of 16
        let n = bit_vector.len() / 32;
        // Process here in chunks of 512 bits - not we're using chunk instead of i
        for chunk in 0..(n / 16) {

            /*
                Create list X from 16 words where a word is 32 bits
                i.e. we should have a vector of 16 32 bit elements
            */
            // Get the beginning chunk
            let begin = chunk * 512;
            // Create X to store our words
            let mut x = Vec::new();
            // Iterate and add to X until we have added 16 32 bit words to X
            for j in 0..16 {
                x.push(&bit_vector[begin + (j * 32)..begin + (j * 32) + 32])
            }
            /*
                Because we use buffer ensure if we have multple chunks (chunk > 1) to use the words
                in 'buffer' from step 3 (buf_clone)
            */

            buffer = Md5Buffer{
                a: buf_clone.a,
                b: buf_clone.b,
                c: buf_clone.c,
                d: buf_clone.d
            };

            // Convert bit values to le integers
            let mut x_int = Vec::new();
            for word in x {
                x_int.push(bin_to_dec(word));
            }
            let mut temp: u32 = 0;
            let mut k = 0;
            let mut s: [u32; 4] = [0, 0, 0, 0];
            let t = StepFour::t_table();

            // Four round execution - each round has 16 operations
            for i in 0..64 {

                let input = AuxInput {
                    x: buffer.b,
                    y: buffer.c,
                    z: buffer.d
                };
                if i <= 15 {
                    k = i;
                    s = [7, 12, 17, 22];
                    temp = StepFour::f(input.clone());
                } else if 16 <= i && i <= 31 {
                    k = ((5 * i) + 1) % 16;
                    s = [5, 9, 14, 20];
                    temp = StepFour::g(input.clone());
                } else if 32 <= i && i <= 47 {
                    k = ((3 * i) + 5) % 16;
                    s = [4, 11, 16, 23];
                    temp = StepFour::h(input.clone());
                } else if 48 <= i && i <= 63 {
                    k = (7 * i) % 16;
                    s = [6, 10, 15, 21];
                    temp = StepFour::i(input.clone());
                }

                /*
                    We use our temp value here so not to overide results but storing them
                    as in putting in A then using A = D will overwrite our result
                */
                temp = temp.wrapping_add(x_int[k] as u32);
                temp = temp.wrapping_add(t[i]);
                temp = temp.wrapping_add(buffer.a);
                temp = StepFour::rotate_left(temp, s[i%4].into());
                temp = temp.wrapping_add(buffer.b);

                // Swap registers for next tick
                /*
                    For each of the four rounds the following takes place:
                    A -> D -> C -> B
                    B -> A -> D -> C
                    C -> B -> A -> D
                    B -> C -> B -> A
                */
                buffer.a = buffer.d;
                buffer.d = buffer.c;
                buffer.c = buffer.b;
                buffer.b = temp;
            }

            // Update the buffers with this chunks results
            buf_clone.a = buf_clone.a.wrapping_add(buffer.a);
            buf_clone.b = buf_clone.b.wrapping_add(buffer.b);
            buf_clone.c = buf_clone.c.wrapping_add(buffer.c);
            buf_clone.d = buf_clone.d.wrapping_add(buffer.d);
        }
        buf_clone

    }
    /*
        Four auxiliary functions to producing one 32 bit word
    */

    /*
        Aux function F is defined as: F(X,Y,Z) = XY v not(X) Z
        in lamens X and Y OR not X and Z - v denotes a bit-wise OR 
    */ 
    fn f(input: AuxInput) -> u32 {
        (input.x & input.y) | (!input.x & input.z)
    }

    // G(X,Y,Z) = XZ v Y not(Z)
    fn g(input: AuxInput) -> u32 {
        (input.x & input.z) | (input.y & !input.z)
    }

    // H(X,Y,Z) = X xor Y xor Z
    fn h(input: AuxInput) -> u32 {
        input.x ^ input.y ^ input.z
    }

    // I(X,Y,Z) = Y xor (X v not(Z))
    fn i(input: AuxInput) -> u32 {
        input.y ^ (input.x | !input.z)
    }

    /*
        Left rotation function which rotates an input 'x' left by 'n' bits
    */
    fn rotate_left(x: u32, n: u32) -> u32 {
        (x << n) | (x >> (32 - n))
    }

    /* 
        Compute T Table (a 64 element table T[1..64]) from the sine function
        "let T[i] denote i-th element, which is equal to int 4294967296 * abs(sin(i))
        where i is in radians"
    */
    fn t_table() -> Vec<u32> {
        let mut t = Vec::new();
        // RFC starts at index 1 so we've done the same, retrieving 64 elements
        for i in 1..65 {
            /*
                t_c (t calculation) is a result of  of 2^32 (as an f64 value here)
                multiplied by the absolute (abs) sine value of i
            */
            let t_c = (2f64.powi(32) * (i as f64).sin().abs()) as u32;
            t.push(t_c);
        }

        // Return our T table
        t
        
    }
}

fn step_five(buffer: Md5Buffer) -> String{
    // 3.5 Step 5. Output - https://tools.ietf.org/html/rfc1321#section-3.5
    /*
        Create as reference four arrays reprsenting A->D with words broken
        to 4 8 bit values (of lowest order)
    */
    let bytes_a : [u8; 4] = (buffer.a as u32).to_le_bytes();
    let bytes_b : [u8; 4] = (buffer.b as u32).to_le_bytes();
    let bytes_c : [u8; 4] = (buffer.c as u32).to_le_bytes();
    let bytes_d : [u8; 4] = (buffer.d as u32).to_le_bytes();

    /*
        Arguably this doesn't have to be the case but it is common for
        output to always be given in hexidecimal.
        Despite me not relying on format! for binary to decimal and back
        as this is a 'choice' on output it felt fine - so take the low
        order of A and follow it through to high of D and return a hex 
        string.
    */
    let msg = format!("{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}\
    {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes_a[0], bytes_a[1], bytes_a[2], bytes_a[3],
            bytes_b[0], bytes_b[1], bytes_b[2], bytes_b[3],
            bytes_c[0], bytes_c[1], bytes_c[2], bytes_c[3],
            bytes_d[0], bytes_d[1], bytes_d[2], bytes_d[3]
        );
    msg
}

#[cfg(test)]
mod test {

    use super::*;

    fn test_hash(input: &String, expect: String) {
        assert_eq!(step_five(StepFour::step_four(step_three(), step_two(step_one(input.as_bytes()), input.to_string()))), expect);
    }
    #[test]
    fn test_testcases() {
        test_hash(&"".to_string(), "d41d8cd98f00b204e9800998ecf8427e".to_string());
        test_hash(&"a".to_string(), "0cc175b9c0f1b6a831c399e269772661".to_string());
        test_hash(&"abc".to_string(), "900150983cd24fb0d6963f7d28e17f72".to_string());
        test_hash(&"message digest".to_string(), "f96b697d7cb7938d525a2f31aaf161d0".to_string());
        test_hash(&"abcdefghijklmnopqrstuvwxyz".to_string(), "c3fcd3d76192e4007dfb496cca67e13b".to_string());
        test_hash(&"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string(), "d174ab98d277d9f5a5611c2c9f419d9f".to_string());
    }
}
