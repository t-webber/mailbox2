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

subject_1: r#"#1 on GitHub. Python Update."# => r#"#1 on GitHub. Python Update."#
subject_2: r#"[scop] some text"# => r#"[scop] some text"#
subject_3: "\t a \n\r" => "a"
subject_4: r#"=?UTF-8?B?76O/IEdQVSBJbnRlcm5zaGlwcyAtIERlc2lnbiBWZXJpZmljYXRpb24gYW5kIEVtdWxhdGlvbiAmIERyaXZlciBMaXZl?="# => r#" GPU Internships - Design Verification and Emulation & Driver Live"#
subject_5: r#"=?UTF-8?B?8J+UtSBJbnZpdGF0aW9uIGF1IHPDqW1pbmFpcmUgZGVzIMOpbMOodmVzIGRlIG5vdHJlIGFzc29jaWF0aW9u?="# => r#"🔵 Invitation au séminaire des élèves de notre association"#
subject_6: r#"=?UTF-8?Q?=C3=89t=C3=A0_fran=C3=A7ais_na=C3=AFf_b=C5=93uf?="# => r#"Étà français naïf bœuf"#
subject_7: r#"=?UTF-8?Q?Invoice_payment_received_=E2=9C=85<>=F0=9F=8C=9F?="# => r#"Invoice payment received ✅<>🌟"#

);
