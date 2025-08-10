// A dead simple deflate decompressor implementation.
// Block type 2 has no optimizations, but it's good enough for my purposes.
//
// Why? Because I don't want to include several dependencies to this project.
// And it's not easy to understand what's happening in zlib only by reading the source code.
use std::collections::HashMap;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Method {
    Stored = 0,
    Fixed = 1,
    Dynamic = 2,
    Invalid = 3,
}

impl Method {
    fn from_u16(n: u16) -> Method {
        match n {
            0 => Method::Stored,
            1 => Method::Fixed,
            2 => Method::Dynamic,
            _ => Method::Invalid,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum LLValue {
    EOB,
    Void,
    Lit(usize, usize),
    Len(usize, usize, usize),
}

use self::LLValue::*;

static CODE_TO_LL_MAP: [LLValue; 512] = [
    EOB,
    Lit(80, 8),
    Lit(16, 8),
    Len(115, 8, 4),
    Len(31, 7, 2),
    Lit(112, 8),
    Lit(48, 8),
    Lit(192, 9),
    Len(10, 7, 0),
    Lit(96, 8),
    Lit(32, 8),
    Lit(160, 9),
    Lit(0, 8),
    Lit(128, 8),
    Lit(64, 8),
    Lit(224, 9),
    Len(6, 7, 0),
    Lit(88, 8),
    Lit(24, 8),
    Lit(144, 9),
    Len(59, 7, 3),
    Lit(120, 8),
    Lit(56, 8),
    Lit(208, 9),
    Len(17, 7, 1),
    Lit(104, 8),
    Lit(40, 8),
    Lit(176, 9),
    Lit(8, 8),
    Lit(136, 8),
    Lit(72, 8),
    Lit(240, 9),
    Len(4, 7, 0),
    Lit(84, 8),
    Lit(20, 8),
    Len(227, 8, 5),
    Len(43, 7, 3),
    Lit(116, 8),
    Lit(52, 8),
    Lit(200, 9),
    Len(13, 7, 1),
    Lit(100, 8),
    Lit(36, 8),
    Lit(168, 9),
    Lit(4, 8),
    Lit(132, 8),
    Lit(68, 8),
    Lit(232, 9),
    Len(8, 7, 0),
    Lit(92, 8),
    Lit(28, 8),
    Lit(152, 9),
    Len(83, 7, 4),
    Lit(124, 8),
    Lit(60, 8),
    Lit(216, 9),
    Len(23, 7, 2),
    Lit(108, 8),
    Lit(44, 8),
    Lit(184, 9),
    Lit(12, 8),
    Lit(140, 8),
    Lit(76, 8),
    Lit(248, 9),
    Len(3, 7, 0),
    Lit(82, 8),
    Lit(18, 8),
    Len(163, 8, 5),
    Len(35, 7, 3),
    Lit(114, 8),
    Lit(50, 8),
    Lit(196, 9),
    Len(11, 7, 1),
    Lit(98, 8),
    Lit(34, 8),
    Lit(164, 9),
    Lit(2, 8),
    Lit(130, 8),
    Lit(66, 8),
    Lit(228, 9),
    Len(7, 7, 0),
    Lit(90, 8),
    Lit(26, 8),
    Lit(148, 9),
    Len(67, 7, 4),
    Lit(122, 8),
    Lit(58, 8),
    Lit(212, 9),
    Len(19, 7, 2),
    Lit(106, 8),
    Lit(42, 8),
    Lit(180, 9),
    Lit(10, 8),
    Lit(138, 8),
    Lit(74, 8),
    Lit(244, 9),
    Len(5, 7, 0),
    Lit(86, 8),
    Lit(22, 8),
    Void,
    Len(51, 7, 3),
    Lit(118, 8),
    Lit(54, 8),
    Lit(204, 9),
    Len(15, 7, 1),
    Lit(102, 8),
    Lit(38, 8),
    Lit(172, 9),
    Lit(6, 8),
    Lit(134, 8),
    Lit(70, 8),
    Lit(236, 9),
    Len(9, 7, 0),
    Lit(94, 8),
    Lit(30, 8),
    Lit(156, 9),
    Len(99, 7, 4),
    Lit(126, 8),
    Lit(62, 8),
    Lit(220, 9),
    Len(27, 7, 2),
    Lit(110, 8),
    Lit(46, 8),
    Lit(188, 9),
    Lit(14, 8),
    Lit(142, 8),
    Lit(78, 8),
    Lit(252, 9),
    Void,
    Lit(81, 8),
    Lit(17, 8),
    Len(131, 8, 5),
    Void,
    Lit(113, 8),
    Lit(49, 8),
    Lit(194, 9),
    Void,
    Lit(97, 8),
    Lit(33, 8),
    Lit(162, 9),
    Lit(1, 8),
    Lit(129, 8),
    Lit(65, 8),
    Lit(226, 9),
    Void,
    Lit(89, 8),
    Lit(25, 8),
    Lit(146, 9),
    Void,
    Lit(121, 8),
    Lit(57, 8),
    Lit(210, 9),
    Void,
    Lit(105, 8),
    Lit(41, 8),
    Lit(178, 9),
    Lit(9, 8),
    Lit(137, 8),
    Lit(73, 8),
    Lit(242, 9),
    Void,
    Lit(85, 8),
    Lit(21, 8),
    Len(258, 8, 0),
    Void,
    Lit(117, 8),
    Lit(53, 8),
    Lit(202, 9),
    Void,
    Lit(101, 8),
    Lit(37, 8),
    Lit(170, 9),
    Lit(5, 8),
    Lit(133, 8),
    Lit(69, 8),
    Lit(234, 9),
    Void,
    Lit(93, 8),
    Lit(29, 8),
    Lit(154, 9),
    Void,
    Lit(125, 8),
    Lit(61, 8),
    Lit(218, 9),
    Void,
    Lit(109, 8),
    Lit(45, 8),
    Lit(186, 9),
    Lit(13, 8),
    Lit(141, 8),
    Lit(77, 8),
    Lit(250, 9),
    Void,
    Lit(83, 8),
    Lit(19, 8),
    Len(195, 8, 5),
    Void,
    Lit(115, 8),
    Lit(51, 8),
    Lit(198, 9),
    Void,
    Lit(99, 8),
    Lit(35, 8),
    Lit(166, 9),
    Lit(3, 8),
    Lit(131, 8),
    Lit(67, 8),
    Lit(230, 9),
    Void,
    Lit(91, 8),
    Lit(27, 8),
    Lit(150, 9),
    Void,
    Lit(123, 8),
    Lit(59, 8),
    Lit(214, 9),
    Void,
    Lit(107, 8),
    Lit(43, 8),
    Lit(182, 9),
    Lit(11, 8),
    Lit(139, 8),
    Lit(75, 8),
    Lit(246, 9),
    Void,
    Lit(87, 8),
    Lit(23, 8),
    Void,
    Void,
    Lit(119, 8),
    Lit(55, 8),
    Lit(206, 9),
    Void,
    Lit(103, 8),
    Lit(39, 8),
    Lit(174, 9),
    Lit(7, 8),
    Lit(135, 8),
    Lit(71, 8),
    Lit(238, 9),
    Void,
    Lit(95, 8),
    Lit(31, 8),
    Lit(158, 9),
    Void,
    Lit(127, 8),
    Lit(63, 8),
    Lit(222, 9),
    Void,
    Lit(111, 8),
    Lit(47, 8),
    Lit(190, 9),
    Lit(15, 8),
    Lit(143, 8),
    Lit(79, 8),
    Lit(254, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(193, 9),
    Void,
    Void,
    Void,
    Lit(161, 9),
    Void,
    Void,
    Void,
    Lit(225, 9),
    Void,
    Void,
    Void,
    Lit(145, 9),
    Void,
    Void,
    Void,
    Lit(209, 9),
    Void,
    Void,
    Void,
    Lit(177, 9),
    Void,
    Void,
    Void,
    Lit(241, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(201, 9),
    Void,
    Void,
    Void,
    Lit(169, 9),
    Void,
    Void,
    Void,
    Lit(233, 9),
    Void,
    Void,
    Void,
    Lit(153, 9),
    Void,
    Void,
    Void,
    Lit(217, 9),
    Void,
    Void,
    Void,
    Lit(185, 9),
    Void,
    Void,
    Void,
    Lit(249, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(197, 9),
    Void,
    Void,
    Void,
    Lit(165, 9),
    Void,
    Void,
    Void,
    Lit(229, 9),
    Void,
    Void,
    Void,
    Lit(149, 9),
    Void,
    Void,
    Void,
    Lit(213, 9),
    Void,
    Void,
    Void,
    Lit(181, 9),
    Void,
    Void,
    Void,
    Lit(245, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(205, 9),
    Void,
    Void,
    Void,
    Lit(173, 9),
    Void,
    Void,
    Void,
    Lit(237, 9),
    Void,
    Void,
    Void,
    Lit(157, 9),
    Void,
    Void,
    Void,
    Lit(221, 9),
    Void,
    Void,
    Void,
    Lit(189, 9),
    Void,
    Void,
    Void,
    Lit(253, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(195, 9),
    Void,
    Void,
    Void,
    Lit(163, 9),
    Void,
    Void,
    Void,
    Lit(227, 9),
    Void,
    Void,
    Void,
    Lit(147, 9),
    Void,
    Void,
    Void,
    Lit(211, 9),
    Void,
    Void,
    Void,
    Lit(179, 9),
    Void,
    Void,
    Void,
    Lit(243, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(203, 9),
    Void,
    Void,
    Void,
    Lit(171, 9),
    Void,
    Void,
    Void,
    Lit(235, 9),
    Void,
    Void,
    Void,
    Lit(155, 9),
    Void,
    Void,
    Void,
    Lit(219, 9),
    Void,
    Void,
    Void,
    Lit(187, 9),
    Void,
    Void,
    Void,
    Lit(251, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(199, 9),
    Void,
    Void,
    Void,
    Lit(167, 9),
    Void,
    Void,
    Void,
    Lit(231, 9),
    Void,
    Void,
    Void,
    Lit(151, 9),
    Void,
    Void,
    Void,
    Lit(215, 9),
    Void,
    Void,
    Void,
    Lit(183, 9),
    Void,
    Void,
    Void,
    Lit(247, 9),
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Void,
    Lit(207, 9),
    Void,
    Void,
    Void,
    Lit(175, 9),
    Void,
    Void,
    Void,
    Lit(239, 9),
    Void,
    Void,
    Void,
    Lit(159, 9),
    Void,
    Void,
    Void,
    Lit(223, 9),
    Void,
    Void,
    Void,
    Lit(191, 9),
    Void,
    Void,
    Void,
    Lit(255, 9),
];

// (dist, extra bits) pair
static SYMBOL_TO_DIST_MAP: [(usize, usize); 32] = [
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 1),
    (7, 1),
    (9, 2),
    (13, 2),
    (17, 3),
    (25, 3),
    (33, 4),
    (49, 4),
    (65, 5),
    (97, 5),
    (129, 6),
    (193, 6),
    (257, 7),
    (385, 7),
    (513, 8),
    (769, 8),
    (1025, 9),
    (1537, 9),
    (2049, 10),
    (3073, 10),
    (4097, 11),
    (6145, 11),
    (8193, 12),
    (12289, 12),
    (16385, 13),
    (24577, 13),
    (0, 0),
    (0, 0),
];

// (length, extra bits) pair
// offset of 257: symbol - 257
static SYMBOL_TO_LL_MAP: [(usize, usize); 32] = [
    (3, 0),
    (4, 0),
    (5, 0),
    (6, 0),
    (7, 0),
    (8, 0),
    (9, 0),
    (10, 0),
    (11, 1),
    (13, 1),
    (15, 1),
    (17, 1),
    (19, 2),
    (23, 2),
    (27, 2),
    (31, 2),
    (35, 3),
    (43, 3),
    (51, 3),
    (59, 3),
    (67, 4),
    (83, 4),
    (99, 4),
    (115, 4),
    (131, 5),
    (163, 5),
    (195, 5),
    (227, 5),
    (258, 0),
    (0, 0),
    (0, 0),
    (0, 0),
];

struct Bitstream {
    data: Vec<u8>,
    pos: usize,

    bit_buff: u32,
    bit_buff_count: usize,
}

impl Bitstream {
    fn new(data: Vec<u8>) -> Self {
        Bitstream {
            data,
            pos: 0,
            bit_buff: 0,
            bit_buff_count: 0,
        }
    }

    fn get_byte(&mut self) -> Result<u8, String> {
        if self.data.len() > self.pos {
            let value = self.data[self.pos];
            self.pos += 1;

            Ok(value)
        } else {
            Err("Could not read byte".to_string())
        }
    }
    fn needbits(&mut self, n: usize) -> Result<(), String> {
        while self.bit_buff_count < n {
            self.bit_buff |= (self.get_byte()? as u32) << self.bit_buff_count;
            self.bit_buff_count += 8;
        }
        Ok(())
    }

    fn dumpbits(&mut self, n: usize) {
        self.bit_buff >>= n;
        self.bit_buff_count -= n;
    }

    fn getbits(&self, n: usize) -> u16 {
        if n == 0 {
            return 0;
        }

        (self.bit_buff & ((1 << n) - 1)) as u16
    }

    fn readbits(&mut self, n: usize) -> Result<u16, String> {
        self.needbits(n)?;
        let v = self.getbits(n);
        self.dumpbits(n);

        Ok(v)
    }
}

fn reverse_u16_bits(mut v: u16, num_bits: usize) -> u16 {
    let mut result = 0;

    for _ in 0..num_bits {
        result = (result << 1) | (v & 1);
        v >>= 1;
    }

    result
}

fn read_block_type1(stream: &mut Bitstream, bytes_window: &mut Vec<u8>) -> Result<(), String> {
    let min_code_length = 7;
    let max_code_length = 9;

    'decode: loop {
        let mut found = false;

        for b in min_code_length..=max_code_length {
            stream.needbits(b)?;

            let v = stream.getbits(b) as usize;
            // let v = reverse_u16_bits(v0, 8) as usize;

            match CODE_TO_LL_MAP[v] {
                Lit(lit, num_bits) => {
                    if num_bits == b {
                        found = true;
                        // println!("lit = '{}', bits = {}, v = {}", (lit as u8) as char, b, v);

                        bytes_window.push(lit as u8);
                        stream.dumpbits(b);
                        break;
                    }
                }
                Len(len, num_bits, extra) => {
                    if num_bits == b {
                        found = true;
                        stream.dumpbits(b);

                        let extra_value = stream.readbits(extra)? as usize;
                        let len2 = len + extra_value;

                        let dist_code = reverse_u16_bits(stream.readbits(5)?, 5) as usize;
                        let (dist, extra2) = SYMBOL_TO_DIST_MAP[dist_code];

                        let extra_value = stream.readbits(extra2)? as usize;
                        let dist2 = dist + extra_value;

                        // println!("len = '{}', dist: {}, bits = {}, v = {}", len2, dist2, b, v);

                        let start = bytes_window.len() - dist2;

                        for k in 0..len2 {
                            bytes_window.push(bytes_window[start + k]);
                        }

                        break;
                    }
                }
                EOB => {
                    stream.dumpbits(b);
                    break 'decode;
                }
                Void => {}
            }
        }

        if !found {
            return Err("literal or length not found".to_string());
        }
    }

    Ok(())
}

fn gen_code_map(code_lens: &[usize]) -> (usize, usize, HashMap<(usize, usize), usize>) {
    // bit_length_count contains the number of codes of each length
    let mut bit_length_count = [0usize; 16];

    // minimum and maximum lengths, boundaries
    let mut min_code_length = 15;
    let mut max_code_length = 0;

    for &code_len in code_lens.iter() {
        if code_len != 0 {
            bit_length_count[code_len] += 1;

            if code_len < min_code_length {
                min_code_length = code_len;
            }

            if code_len > max_code_length {
                max_code_length = code_len;
            }
        }
    }

    // next_codes contains the next code of each bit length
    let mut next_codes = [0usize; 17];

    let mut code = 0;
    for i in 1..bit_length_count.len() {
        code = (code + bit_length_count[i - 1]) << 1;
        next_codes[i] = code;
    }

    // println!("next_codes = {:?}", next_codes);

    let mut map = HashMap::new();

    for i in 0..code_lens.len() {
        let code_len = code_lens[i];

        if code_len != 0 {
            let code = reverse_u16_bits(next_codes[code_len] as u16, code_len) as usize;

            // println!("code_len = {}, code = {:0>w$b}", i, code, w = code_len);
            map.insert((code, code_len), i);

            next_codes[code_len] += 1;
        }
    }

    (min_code_length, max_code_length, map)
}

fn read_block_type2(stream: &mut Bitstream, bytes_window: &mut Vec<u8>) -> Result<(), String> {
    let hlit = stream.readbits(5)?;
    let num_lit_len_codes = (hlit + 257) as usize;

    // println!("num_lit_len_codes = {}", num_lit_len_codes);

    let hdist = stream.readbits(5)?;
    let num_dist_codes = (hdist + 1) as usize;

    // println!("num_dist_codes = {}", num_dist_codes);

    let hclen = stream.readbits(4)?;
    let num_code_len_codes = (hclen + 4) as usize;

    // println!("num_code_len_codes = {}", num_code_len_codes);

    let cl_code_len_order: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut cl_code_lens: [usize; 19] = [0usize; 19];

    for i in 0..num_code_len_codes {
        let cl_code_len = stream.readbits(3)? as usize;
        let pos = cl_code_len_order[i];
        cl_code_lens[pos] = cl_code_len;
    }

    // println!("cl_code_lens = {:?}", cl_code_lens);

    let (min_cl_code_length, max_cl_code_length, cl_code_map) = gen_code_map(&cl_code_lens);

    // println!("min_cl_code_len = {}", min_cl_code_length);
    // println!("max_cl_code_len = {}", max_cl_code_length);

    let mut ll_code_lens = [0usize; 288];
    let mut last_code = 0;
    let mut i = 0usize;

    while i < num_lit_len_codes {
        let mut found = false;

        for b in min_cl_code_length..=max_cl_code_length {
            stream.needbits(b)?;

            let v = stream.getbits(b) as usize;

            if let Some(&cl_code_len) = cl_code_map.get(&(v, b)) {
                // println!("cl_code_len = {}, bits = {}, v = {}", cl_code_len, b, v);
                stream.dumpbits(b);

                if cl_code_len <= 15 {
                    ll_code_lens[i] = cl_code_len;
                    last_code = cl_code_len;

                    // println!("  code len '{}'", last_code);
                    i += 1;
                } else if cl_code_len == 16 {
                    let repeat = 3 + stream.readbits(2)? as usize;

                    for k in 0..repeat {
                        ll_code_lens[i + k] = last_code;
                    }

                    i += repeat as usize;

                    // println!("  repeat '{}' {} times", last_code, repeat);
                } else if cl_code_len == 17 {
                    let repeat = 3 + stream.readbits(3)? as usize;

                    i += repeat as usize;
                    // println!("  repeat '0' {} times", repeat);
                } else if cl_code_len == 18 {
                    let repeat = 11 + stream.readbits(7)? as usize;

                    i += repeat as usize;
                    // println!("  repeat '0' {} times", repeat);
                } else {
                    unreachable!();
                }

                found = true;
                break;
            }
        }

        if !found {
            return Err("cl_code_len not found".to_string());
        }
    }

    if i != num_lit_len_codes {
        return Err(format!(
            "Expected {} lit_len codes, got {}",
            num_lit_len_codes, i
        ));
    }

    let (min_ll_code_len, max_ll_code_len, ll_code_map) = gen_code_map(&ll_code_lens);

    // println!("min_ll_code_len = {}", min_ll_code_len);
    // println!("max_ll_code_len = {}", max_ll_code_len);

    let mut dist_code_lens = [0usize; 32];

    i = 0;

    while i < num_dist_codes {
        let mut found = false;

        for b in min_cl_code_length..=max_cl_code_length {
            stream.needbits(b)?;

            let v = stream.getbits(b) as usize;

            if let Some(&cl_code_len) = cl_code_map.get(&(v, b)) {
                // println!("cl_code_len = {}, bits = {}, v = {}", cl_code_len, b, v);
                stream.dumpbits(b);

                if cl_code_len <= 15 {
                    dist_code_lens[i] = cl_code_len;
                    last_code = cl_code_len;

                    // println!("  code len '{}'", last_code);
                    i += 1;
                } else if cl_code_len == 16 {
                    let repeat = 3 + stream.readbits(2)? as usize;

                    for k in 0..repeat {
                        dist_code_lens[i + k] = last_code;
                    }

                    i += repeat as usize;

                    // println!("  repeat '{}' {} times", last_code, repeat);
                } else if cl_code_len == 17 {
                    let repeat = 3 + stream.readbits(3)? as usize;

                    i += repeat as usize;
                    // println!("  repeat '0' {} times", repeat);
                } else if cl_code_len == 18 {
                    let repeat = 11 + stream.readbits(7)? as usize;

                    i += repeat as usize;
                    // println!("  repeat '0' {} times", repeat);
                } else {
                    unreachable!();
                }

                found = true;
                break;
            }
        }

        if !found {
            return Err("cl_code_len not found".to_string());
        }
    }

    assert_eq!(i, num_dist_codes);

    let (min_dist_code_len, max_dist_code_len, dist_code_map) = gen_code_map(&dist_code_lens);

    'decode: loop {
        let mut found = false;
        for b in min_ll_code_len..=max_ll_code_len {
            stream.needbits(b)?;

            let v = stream.getbits(b) as usize;

            if let Some(&symbol) = ll_code_map.get(&(v, b)) {
                found = true;
                stream.dumpbits(b);

                if symbol < 256 {
                    // println!("lit = '{}', bits = {}, v = {}", (symbol as u8) as char, b, v);

                    bytes_window.push(symbol as u8);
                } else if symbol == 256 {
                    break 'decode;
                } else {
                    let (len, extra_bits) = SYMBOL_TO_LL_MAP[symbol - 257];
                    let extra_value = stream.readbits(extra_bits)? as usize;
                    let len2 = len + extra_value;

                    let mut found2 = false;
                    for b2 in min_dist_code_len..=max_dist_code_len {
                        stream.needbits(b2)?;

                        let v2 = stream.getbits(b2) as usize;

                        if let Some(&dist_symbol) = dist_code_map.get(&(v2, b2)) {
                            found2 = true;
                            stream.dumpbits(b2);

                            let (dist, extra_bits) = SYMBOL_TO_DIST_MAP[dist_symbol];
                            let extra_value = stream.readbits(extra_bits)? as usize;
                            let dist2 = dist + extra_value;

                            // println!("len = '{}', dist: {}, bits = {}, v = {}, bits2 = {}, v2 = {}, window len = {}", len2, dist2, b, v, b2, v2, bytes_window.len());
                            let start = bytes_window.len() - dist2;
                            for k in 0..len2 {
                                bytes_window.push(bytes_window[start + k]);
                            }

                            break;
                        }
                    }

                    if !found2 {
                        return Err("dist_code not found".to_string());
                    }
                }

                break;
            }
        }

        if !found {
            return Err("lit_len not found".to_string());
        }
    }

    Ok(())
}

fn read_block_type0(stream: &mut Bitstream, bytes_window: &mut Vec<u8>) -> Result<(), String> {
    // These bits are ignored
    let _ = stream.readbits(5)?;

    let len = stream.readbits(16)?;
    let nlen = stream.readbits(16)?;

    assert_eq!(len, !nlen);

    for _ in 0..len {
        let byte = stream.readbits(8)? as u8;
        bytes_window.push(byte);
    }

    Ok(())
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut stream = Bitstream::new(data.to_vec());
    let mut bytes_window = Vec::new();

    loop {
        let last_block = stream.readbits(1)? == 1;

        let method = Method::from_u16(stream.readbits(2)?);

        match method {
            Method::Stored => {
                read_block_type0(&mut stream, &mut bytes_window)?;
            }
            Method::Fixed => {
                read_block_type1(&mut stream, &mut bytes_window)?;
            }
            Method::Dynamic => {
                read_block_type2(&mut stream, &mut bytes_window)?;
            }
            Method::Invalid => unreachable!(),
        }

        if last_block {
            break;
        }
    }

    Ok(bytes_window)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_block_type0() {
        let data = vec![1, 8, 0, 247, 255, 82, 97, 119, 32, 68, 97, 116, 97];
        let bytes_window = decompress(&data).unwrap();
        let text = "Raw Data";
        assert_eq!(str::from_utf8(&bytes_window[..]).unwrap(), text.to_string());
    }

    #[test]
    fn test_decompress_block_type1() {
        let data = vec![
            243, 72, 205, 201, 201, 215, 81, 240, 192, 70, 85, 101, 22, 40, 114, 1, 0,
        ];

        let bytes_window = decompress(&data).unwrap();
        let text = "Hello, Hello, Hello, Hello, zip!\n";

        assert_eq!(str::from_utf8(&bytes_window[..]).unwrap(), text.to_string());
    }

    #[test]
    fn test_decompress_block_type2() {
        let data = vec![
            213, 143, 49, 82, 68, 33, 16, 68, 115, 78, 209, 153, 137, 151, 208, 140, 196, 68, 61,
            0, 43, 243, 63, 212, 2, 67, 193, 80, 236, 191, 189, 195, 95, 215, 19, 152, 88, 53, 9,
            211, 205, 244, 107, 43, 232, 226, 154, 144, 199, 140, 18, 224, 144, 120, 34, 197, 61,
            200, 179, 121, 163, 155, 64, 66, 44, 59, 44, 174, 133, 166, 62, 232, 64, 139, 181, 170,
            127, 107, 156, 145, 15, 92, 200, 155, 151, 226, 151, 86, 238, 6, 97, 190, 158, 74, 98,
            214, 253, 81, 201, 88, 65, 162, 77, 244, 126, 151, 230, 202, 78, 136, 185, 54, 234, 61,
            114, 129, 142, 186, 3, 57, 111, 140, 197, 116, 189, 60, 41, 214, 104, 132, 25, 156, 18,
            48, 60, 155, 215, 33, 15, 10, 139, 224, 252, 125, 141, 206, 153, 78, 68, 243, 206, 42,
            156, 209, 14, 158, 168, 226, 210, 200, 73, 56, 217, 236, 111, 75, 253, 213, 70, 89, 57,
            138, 129, 81, 36, 38, 85, 191, 120, 36, 191, 82, 93, 57, 50, 55, 50, 31, 171, 204, 31,
            120, 62, 127, 132, 101, 98, 109, 192, 219, 3, 235, 63, 53, 53, 223,
        ];

        let bytes_window = decompress(&data).unwrap();
        let text = "It started with a low light,
Next thing I knew they ripped from my bed
And then they took my blood type
It left a strange impression on my head

I wasn't sure what to do
But I knew I had to do something
So I took a deep breath
And I started to run

I ran until I couldn't anymore
Then I ran until I couldn't anymore
Then I ran until I couldn't anymore
Until I ran out of breath

I wasn't sure what to do
But I knew I had to do something
So I took a deep breath
And I started to run

I ran until I couldn't anymore
Then I ran until I couldn't anymore

";

        assert_eq!(str::from_utf8(&bytes_window[..]).unwrap(), text.to_string());
    }
}
