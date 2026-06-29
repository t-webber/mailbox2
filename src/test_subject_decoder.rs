use crate::subject_decoder::decode_subject;

macro_rules! tst {
    ($($name:ident: $in:expr => $out:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(decode_subject($in), $out);
            }
        )*
    };
}

tst!(

subject_1: "#1 on GitHub. Python Update." => "#1 on GitHub. Python Update."
subject_2: "[scop] some text" => "[scop] some text"
subject_3: "\t a \n\r" => "a"
subject_4: "=?UTF-8?B?76O/IEdQVSBJbnRlcm5zaGlwcyAtIERlc2lnbiBWZXJpZmljYXRpb24gYW5kIEVtdWxhdGlvbiAmIERyaXZlciBMaXZl?=" => "\u{f8ff} GPU Internships - Design Verification and Emulation & Driver Live"
subject_5: "=?UTF-8?B?8J+UtSBJbnZpdGF0aW9uIGF1IHPDqW1pbmFpcmUgZGVzIMOpbMOodmVzIGRlIG5vdHJlIGFzc29jaWF0aW9u?=" => "\u{1f535} Invitation au s\u{e9}minaire des \u{e9}l\u{e8}ves de notre association"
subject_6: "=?UTF-8?Q?=C3=89t=C3=A0_fran=C3=A7ais_na=C3=AFf_b=C5=93uf?=" => "\u{c9}t\u{e0} fran\u{e7}ais na\u{ef}f b\u{153}uf"
subject_7: "=?UTF-8?Q?Invoice_payment_received_=E2=9C=85<>=F0=9F=8C=9F?=" => "Invoice payment received \u{2705}<>\u{1f31f}"

);
