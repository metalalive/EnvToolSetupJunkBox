// note the macro `link` can be omitted cuz it helps only
// linking resolution, it does not cuase performance impact

#[link(name = "eexampaul")]
extern {
    fn censor_packet(p:* mut HighLvlPkt) -> i32;
    fn buggy_cross_product(v1:*mut i8, v2:*const i8, num_cols:u8) -> i32;
}

#[derive(Debug)]
#[repr(C)]
struct HighLvlPktPayload {
	data:* mut u8,
	len:usize,
}
#[derive(Debug)]
#[repr(C)]
struct HighLvlPkt {
    flags:u8,
    timeout_s:u16,
    qos:u8,
    fastopen:u8,
    payld:HighLvlPktPayload,
}

fn main() {
    println!("------- foreign function interface demo ------");
    let mut message = "(M@r*3d4#J:14iwth3498ty".to_string();
    let orig_msg_sz = message.len();
    let after_msg_sz = orig_msg_sz - 5;
    let mut exp_pkt = HighLvlPkt {
        flags:0x95u8,  timeout_s:77u16, qos:2u8, fastopen:13u8,
        payld:HighLvlPktPayload{len: orig_msg_sz,  data:message.as_mut_ptr()},
    };
    let mut myvec1:Vec<i8> = vec![27, -11, 29, -15,  4];
    let myvec2:Vec<i8> = vec![-1, -1,   1,   0, -1];
    let out1:i32 = unsafe{ censor_packet(&mut exp_pkt as * mut HighLvlPkt) };
    let out2:i32 = unsafe{
        buggy_cross_product(myvec1.as_mut_ptr(), myvec2.as_ptr(), 5u8)
    };
    assert_eq!(out1,  0i32);
    assert_eq!(out2,  9 );
    assert_eq!(myvec1, [26,-12,28,-16,3]);
    println!("packet after analysis --- {:?}", exp_pkt);
    assert_eq!(exp_pkt.fastopen,  26u8);
    assert_eq!(exp_pkt.flags,  0x9au8);
    assert_eq!(exp_pkt.timeout_s,  77u16);
    assert_eq!(exp_pkt.payld.len, after_msg_sz);
    assert!(message.starts_with("Mr3d4J14iwth3498ty"));
}
