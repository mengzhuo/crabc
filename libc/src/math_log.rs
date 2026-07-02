// Translated from musl: log.c, logf.c, log10.c, log10f.c, log2.c, log2f.c
// and their associated *_data.c files.

// ============================================================
// log (double precision) data
// ============================================================

const LOG_TABLE_BITS: u32 = 7;
const LOG_N: usize = 1 << LOG_TABLE_BITS;
const LN2HI: f64 = asdouble(0x3fe62e42fefa3800);
const LN2LO: f64 = asdouble(0x3d2ef35793c76730);

const LOG_POLY1: [f64; 11] = [
    asdouble(0xbfe0000000000000),
    asdouble(0x3fd5555555555577),
    asdouble(0xbfcffffffffffdcb),
    asdouble(0x3fc999999995dd0c),
    asdouble(0xbfc55555556745a7),
    asdouble(0x3fc24924a344de30),
    asdouble(0xbfbfffffa4423d65),
    asdouble(0x3fbc7184282ad6ca),
    asdouble(0xbfb999eb43b068ff),
    asdouble(0x3fb78182f7afd085),
    asdouble(0xbfb5521375d145cd),
];

const LOG_POLY: [f64; 5] = [
    asdouble(0xbfe0000000000001),
    asdouble(0x3fd555555551305b),
    asdouble(0xbfcfffffffeb4590),
    asdouble(0x3fc999b324f10111),
    asdouble(0xbfc55575e506c89f),
];

const LOG_TAB: [(u64, u64); 128] = [
    (0x3ff734f0c3e0de9f, 0xbfd7cc7f79e69000),
    (0x3ff713786a2ce91f, 0xbfd76feec20d0000),
    (0x3ff6f26008fab5a0, 0xbfd713e31351e000),
    (0x3ff6d1a61f138c7d, 0xbfd6b85b38287800),
    (0x3ff6b1490bc5b4d1, 0xbfd65d5590807800),
    (0x3ff69147332f0cba, 0xbfd602d076180000),
    (0x3ff6719f18224223, 0xbfd5a8ca86909000),
    (0x3ff6524f99a51ed9, 0xbfd54f4356035000),
    (0x3ff63356aa8f24c4, 0xbfd4f637c36b4000),
    (0x3ff614b36b9ddc14, 0xbfd49da7fda85000),
    (0x3ff5f66452c65c4c, 0xbfd445923989a800),
    (0x3ff5d867b5912c4f, 0xbfd3edf439b0b800),
    (0x3ff5babccb5b90de, 0xbfd396ce448f7000),
    (0x3ff59d61f2d91a78, 0xbfd3401e17bda000),
    (0x3ff5805612465687, 0xbfd2e9e2ef468000),
    (0x3ff56397cee76bd3, 0xbfd2941b3830e000),
    (0x3ff54725e2a77f93, 0xbfd23ec58cda8800),
    (0x3ff52aff42064583, 0xbfd1e9e129279000),
    (0x3ff50f22dbb2bddf, 0xbfd1956d2b48f800),
    (0x3ff4f38f4734ded7, 0xbfd141679ab9f800),
    (0x3ff4d843cfde2840, 0xbfd0edd094ef9800),
    (0x3ff4bd3ec078a3c8, 0xbfd09aa518db1000),
    (0x3ff4a27fc3e0258a, 0xbfd047e65263b800),
    (0x3ff4880524d48434, 0xbfcfeb224586f000),
    (0x3ff46dce1b192d0b, 0xbfcf474a7517b000),
    (0x3ff453d9d3391854, 0xbfcea4443d103000),
    (0x3ff43a2744b4845a, 0xbfce020d44e9b000),
    (0x3ff420b54115f8fb, 0xbfcd60a22977f000),
    (0x3ff40782da3ef4b1, 0xbfccc00104959000),
    (0x3ff3ee8f5d57fe8f, 0xbfcc202956891000),
    (0x3ff3d5d9a00b4ce9, 0xbfcb81178d811000),
    (0x3ff3bd60c010c12b, 0xbfcae2c9ccd3d000),
    (0x3ff3a5242b75dab8, 0xbfca45402e129000),
    (0x3ff38d22cd9fd002, 0xbfc9a877681df000),
    (0x3ff3755bc5847a1c, 0xbfc90c6d69483000),
    (0x3ff35dce49ad36e2, 0xbfc87120a645c000),
    (0x3ff34679984dd440, 0xbfc7d68fb4143000),
    (0x3ff32f5cceffcb24, 0xbfc73cb83c627000),
    (0x3ff3187775a10d49, 0xbfc6a39a9b376000),
    (0x3ff301c8373e3990, 0xbfc60b3154b7a000),
    (0x3ff2eb4ebb95f841, 0xbfc5737d76243000),
    (0x3ff2d50a0219a9d1, 0xbfc4dc7b8fc23000),
    (0x3ff2bef9a8b7fd2a, 0xbfc4462c51d20000),
    (0x3ff2a91c7a0c1bab, 0xbfc3b08abc830000),
    (0x3ff293726014b530, 0xbfc31b996b490000),
    (0x3ff27dfa5757a1f5, 0xbfc2875490a44000),
    (0x3ff268b39b1d3bbf, 0xbfc1f3b9f879a000),
    (0x3ff2539d838ff5bd, 0xbfc160c8252ca000),
    (0x3ff23eb7aac9083b, 0xbfc0ce7f57f72000),
    (0x3ff22a012ba940b6, 0xbfc03cdc49fea000),
    (0x3ff2157996cc4132, 0xbfbf57bdbc4b8000),
    (0x3ff201201dd2fc9b, 0xbfbe370896404000),
    (0x3ff1ecf4494d480b, 0xbfbd17983ef94000),
    (0x3ff1d8f5528f6569, 0xbfbbf9674ed8a000),
    (0x3ff1c52311577e7c, 0xbfbadc79202f6000),
    (0x3ff1b17c74cb26e9, 0xbfb9c0c3e7288000),
    (0x3ff19e010c2c1ab6, 0xbfb8a646b372c000),
    (0x3ff18ab07bb670bd, 0xbfb78d01b3ac0000),
    (0x3ff1778a25efbcb6, 0xbfb674f145380000),
    (0x3ff1648d354c31da, 0xbfb55e0e6d878000),
    (0x3ff151b990275fdd, 0xbfb4485cdea1e000),
    (0x3ff13f0ea432d24c, 0xbfb333d94d6aa000),
    (0x3ff12c8b7210f9da, 0xbfb22079f8c56000),
    (0x3ff11a3028ecb531, 0xbfb10e4698622000),
    (0x3ff107fbda8434af, 0xbfaffa6c6ad20000),
    (0x3ff0f5ee0f4e6bb3, 0xbfadda8d4a774000),
    (0x3ff0e4065d2a9fce, 0xbfabbcece4850000),
    (0x3ff0d244632ca521, 0xbfa9a1894012c000),
    (0x3ff0c0a77ce2981a, 0xbfa788583302c000),
    (0x3ff0af2f83c636d1, 0xbfa5715e67d68000),
    (0x3ff09ddb98a01339, 0xbfa35c8a49658000),
    (0x3ff08cabaf52e7df, 0xbfa149e364154000),
    (0x3ff07b9f2f4e28fb, 0xbf9e72c082eb8000),
    (0x3ff06ab58c358f19, 0xbf9a55f152528000),
    (0x3ff059eea5ecf92c, 0xbf963d62cf818000),
    (0x3ff04949cdd12c90, 0xbf9228fb8caa0000),
    (0x3ff038c6c6f0ada9, 0xbf8c317b20f90000),
    (0x3ff02865137932a9, 0xbf8419355daa0000),
    (0x3ff0182427ea7348, 0xbf781203c2ec0000),
    (0x3ff008040614b195, 0xbf60040979240000),
    (0x3fefe01ff726fa1a, 0x3f6feff384900000),
    (0x3fefa11cc261ea74, 0x3f87dc41353d0000),
    (0x3fef6310b081992e, 0x3f93cea3c4c28000),
    (0x3fef25f63ceeadcd, 0x3f9b9fc114890000),
    (0x3feee9c8039113e7, 0x3fa1b0d8ce110000),
    (0x3feeae8078cbb1ab, 0x3fa58a5bd001c000),
    (0x3fee741aa29d0c9b, 0x3fa95c8340d88000),
    (0x3fee3a91830a99b5, 0x3fad276aef578000),
    (0x3fee01e009609a56, 0x3fb07598e598c000),
    (0x3fedca01e577bb98, 0x3fb253f5e30d2000),
    (0x3fed92f20b7c9103, 0x3fb42edd8b380000),
    (0x3fed5cac66fb5cce, 0x3fb606598757c000),
    (0x3fed272caa5ede9d, 0x3fb7da76356a0000),
    (0x3fecf26e3e6b2ccd, 0x3fb9ab434e1c6000),
    (0x3fecbe6da2a77902, 0x3fbb78c7bb0d6000),
    (0x3fec8b266d37086d, 0x3fbd431332e72000),
    (0x3fec5894bd5d5804, 0x3fbf0a3171de6000),
    (0x3fec26b533bb9f8c, 0x3fc067152b914000),
    (0x3febf583eeece73f, 0x3fc147858292b000),
    (0x3febc4fd75db96c1, 0x3fc2266ecdca3000),
    (0x3feb951e0c864a28, 0x3fc303d7a6c55000),
    (0x3feb65e2c5ef3e2c, 0x3fc3dfc33c331000),
    (0x3feb374867c9888b, 0x3fc4ba366b7a8000),
    (0x3feb094b211d304a, 0x3fc5933928d1f000),
    (0x3feadbe885f2ef7e, 0x3fc66acd2418f000),
    (0x3feaaf1d31603da2, 0x3fc740f8ec669000),
    (0x3fea82e63fd358a7, 0x3fc815c0f51af000),
    (0x3fea5740ef09738b, 0x3fc8e92954f68000),
    (0x3fea2c2a90ab4b27, 0x3fc9bb3602f84000),
    (0x3fea01a01393f2d1, 0x3fca8bed1c2c0000),
    (0x3fe9d79f24db3c1b, 0x3fcb5b515c01d000),
    (0x3fe9ae2505c7b190, 0x3fcc2967ccbcc000),
    (0x3fe9852ef297ce2f, 0x3fccf635d5486000),
    (0x3fe95cbaeea44b75, 0x3fcdc1bd3446c000),
    (0x3fe934c69de74838, 0x3fce8c01b8cfe000),
    (0x3fe90d4f2f6752e6, 0x3fcf5509c0179000),
    (0x3fe8e6528effd79d, 0x3fd00e6c121fb800),
    (0x3fe8bfce9fcc007c, 0x3fd071b80e93d000),
    (0x3fe899c0dabec30e, 0x3fd0d46b9e867000),
    (0x3fe87427aa2317fb, 0x3fd13687334bd000),
    (0x3fe84f00acb39a08, 0x3fd1980d67234800),
    (0x3fe82a49e8653e55, 0x3fd1f8ffe0cc8000),
    (0x3fe8060195f40260, 0x3fd2595fd7636800),
    (0x3fe7e22563e0a329, 0x3fd2b9300914a800),
    (0x3fe7beb377dcb5ad, 0x3fd3187210436000),
    (0x3fe79baa679725c2, 0x3fd377266dec1800),
    (0x3fe77907f2170657, 0x3fd3d54ffbaf3000),
    (0x3fe756cadbd6130c, 0x3fd432eee32fe000),
];

const LOG_TAB2: [(u64, u64); 128] = [
    (0x3fe61000014fb66b, 0x3c7e026c91425b3c),
    (0x3fe63000034db495, 0x3c8dbfea48005d41),
    (0x3fe650000d94d478, 0x3c8e7fa786d6a5b7),
    (0x3fe67000074e6fad, 0x3c61fcea6b54254c),
    (0x3fe68ffffedf0fae, 0xbc7c7e274c590efd),
    (0x3fe6b0000763c5bc, 0xbc8ac16848dcda01),
    (0x3fe6d0001e5cc1f6, 0x3c833f1c9d499311),
    (0x3fe6efffeb05f63e, 0xbc7e80041ae22d53),
    (0x3fe710000e869780, 0x3c7bff6671097952),
    (0x3fe72ffffc67e912, 0x3c8c00e226bd8724),
    (0x3fe74fffdf81116a, 0xbc6e02916ef101d2),
    (0x3fe770000f679c90, 0xbc67fc71cd549c74),
    (0x3fe78ffffa7ec835, 0x3c81bec19ef50483),
    (0x3fe7affffe20c2e6, 0xbc707e1729cc6465),
    (0x3fe7cfffed3fc900, 0xbc808072087b8b1c),
    (0x3fe7efffe9261a76, 0x3c8dc0286d9df9ae),
    (0x3fe81000049ca3e8, 0x3c897fd251e54c33),
    (0x3fe8300017932c8f, 0xbc8afee9b630f381),
    (0x3fe850000633739c, 0x3c89bfbf6b6535bc),
    (0x3fe87000204289c6, 0xbc8bbf65f3117b75),
    (0x3fe88fffebf57904, 0xbc89006ea23dcb57),
    (0x3fe8b00022bc04df, 0xbc7d00df38e04b0a),
    (0x3fe8cfffe50c1b8a, 0xbc88007146ff9f05),
    (0x3fe8effffc918e43, 0x3c83817bd07a7038),
    (0x3fe910001efa5fc7, 0x3c893e9176dfb403),
    (0x3fe9300013467bb9, 0x3c7f804e4b980276),
    (0x3fe94fffe6ee076f, 0xbc8f7ef0d9ff622e),
    (0x3fe96fffde3c12d1, 0xbc7082aa962638ba),
    (0x3fe98ffff4458a0d, 0xbc87801b9164a8ef),
    (0x3fe9afffdd982e3e, 0xbc8740e08a5a9337),
    (0x3fe9cfffed49fb66, 0x3c3fce08c19be000),
    (0x3fe9f00020f19c51, 0xbc8a3faa27885b0a),
    (0x3fea10001145b006, 0x3c74ff489958da56),
    (0x3fea300007bbf6fa, 0x3c8cbeab8a2b6d18),
    (0x3fea500010971d79, 0x3c88fecadd787930),
    (0x3fea70001df52e48, 0xbc8f41763dd8abdb),
    (0x3fea90001c593352, 0xbc8ebf0284c27612),
    (0x3feab0002a4f3e4b, 0xbc69fd043cff3f5f),
    (0x3feacfffd7ae1ed1, 0xbc823ee7129070b4),
    (0x3feaefffee510478, 0x3c6a063ee00edea3),
    (0x3feb0fffdb650d5b, 0x3c5a06c8381f0ab9),
    (0x3feb2ffffeaaca57, 0xbc79011e74233c1d),
    (0x3feb4fffd995badc, 0xbc79ff1068862a9f),
    (0x3feb7000249e659c, 0x3c8aff45d0864f3e),
    (0x3feb8ffff9871640, 0x3c7cfe7796c2c3f9),
    (0x3febafffd204cb4f, 0xbc63ff27eef22bc4),
    (0x3febcfffd2415c45, 0xbc6cffb7ee3bea21),
    (0x3febeffff86309df, 0xbc814103972e0b5c),
    (0x3fec0fffe1b57653, 0x3c8bc16494b76a19),
    (0x3fec2ffff1fa57e3, 0xbc64feef8d30c6ed),
    (0x3fec4fffdcbfe424, 0xbc843f68bcec4775),
    (0x3fec6fffed54b9f7, 0x3c847ea3f053e0ec),
    (0x3fec8fffeb998fd5, 0x3c7383068df992f1),
    (0x3fecb0002125219a, 0xbc68fd8e64180e04),
    (0x3feccfffdd94469c, 0x3c8e7ebe1cc7ea72),
    (0x3fecefffeafdc476, 0x3c8ebe39ad9f88fe),
    (0x3fed1000169af82b, 0x3c757d91a8b95a71),
    (0x3fed30000d0ff71d, 0x3c89c1906970c7da),
    (0x3fed4fffea790fc4, 0xbc580e37c558fe0c),
    (0x3fed70002edc87e5, 0xbc7f80d64dc10f44),
    (0x3fed900021dc82aa, 0xbc747c8f94fd5c5c),
    (0x3fedafffd86b0283, 0x3c8c7f1dc521617e),
    (0x3fedd000296c4739, 0x3c88019eb2ffb153),
    (0x3fedefffe54490f5, 0x3c6e00d2c652cc89),
    (0x3fee0fffcdabf694, 0xbc7f8340202d69d2),
    (0x3fee2fffdb52c8dd, 0x3c7b00c1ca1b0864),
    (0x3fee4ffff24216ef, 0x3c72ffa8b094ab51),
    (0x3fee6fffe88a5e11, 0xbc57f673b1efbe59),
    (0x3fee9000119eff0d, 0xbc84808d5e0bc801),
    (0x3feeafffdfa51744, 0x3c780006d54320b5),
    (0x3feed0001a127fa1, 0xbc5002f860565c92),
    (0x3feef00007babcc4, 0xbc8540445d35e611),
    (0x3fef0ffff57a8d02, 0xbc4ffb3139ef9105),
    (0x3fef30001ee58ac7, 0x3c8a81acf2731155),
    (0x3fef4ffff5823494, 0x3c8a3f41d4d7c743),
    (0x3fef6ffffca94c6b, 0xbc6202f41c987875),
    (0x3fef8fffe1f9c441, 0x3c777dd1f477e74b),
    (0x3fefafffd2e0e37e, 0xbc6f01199a7ca331),
    (0x3fefd0001c77e49e, 0x3c7181ee4bceacb1),
    (0x3fefeffff7e0c331, 0xbc6e05370170875a),
    (0x3ff00ffff465606e, 0xbc8a7ead491c0ada),
    (0x3ff02ffff3867a58, 0xbc977f69c3fcb2e0),
    (0x3ff04ffffdfc0d17, 0x3c97bffe34cb945b),
    (0x3ff0700003cd4d82, 0x3c820083c0e456cb),
    (0x3ff08ffff9f2cbe8, 0xbc6dffdfbe37751a),
    (0x3ff0b000010cda65, 0xbc913f7faee626eb),
    (0x3ff0d00001a4d338, 0x3c807dfa79489ff7),
    (0x3ff0effffadafdfd, 0xbc77040570d66bc0),
    (0x3ff110000bbafd96, 0x3c8e80d4846d0b62),
    (0x3ff12ffffae5f45d, 0x3c9dbffa64fd36ef),
    (0x3ff150000dd59ad9, 0x3c9a0077701250ae),
    (0x3ff170000f21559a, 0x3c8dfdf9e2e3deee),
    (0x3ff18ffffc275426, 0x3c910030dc3b7273),
    (0x3ff1b000123d3c59, 0x3c997f7980030188),
    (0x3ff1cffff8299eb7, 0xbc65f932ab9f8c67),
    (0x3ff1effff48ad400, 0x3c937fbf9da75beb),
    (0x3ff210000c8b86a4, 0x3c9f806b91fd5b22),
    (0x3ff2300003854303, 0x3c93ffc2eb9fbf33),
    (0x3ff24fffffbcf684, 0x3c7601e77e2e2e72),
    (0x3ff26ffff52921d9, 0x3c7ffcbb767f0c61),
    (0x3ff2900014933a3c, 0xbc7202ca3c02412b),
    (0x3ff2b00014556313, 0xbc92808233f21f02),
    (0x3ff2cfffebfe523b, 0xbc88ff7e384fdcf2),
    (0x3ff2f0000bb8ad96, 0xbc85ff51503041c5),
    (0x3ff30ffffb7ae2af, 0xbc810071885e289d),
    (0x3ff32ffffeac5f7f, 0xbc91ff5d3fb7b715),
    (0x3ff350000ca66756, 0x3c957f82228b82bd),
    (0x3ff3700011fbf721, 0x3c8000bac40dd5cc),
    (0x3ff38ffff9592fb9, 0xbc943f9d2db2a751),
    (0x3ff3b00004ddd242, 0x3c857f6b707638e1),
    (0x3ff3cffff5b2c957, 0x3c7a023a10bf1231),
    (0x3ff3efffeab0b418, 0x3c987f6d66b152b0),
    (0x3ff410001532aff4, 0x3c67f8375f198524),
    (0x3ff4300017478b29, 0x3c8301e672dc5143),
    (0x3ff44fffe795b463, 0x3c89ff69b8b2895a),
    (0x3ff46fffe80475e0, 0xbc95c0b19bc2f254),
    (0x3ff48fffef6fc1e7, 0x3c9b4009f23a2a72),
    (0x3ff4afffe5bea704, 0xbc94ffb7bf0d7d45),
    (0x3ff4d000171027de, 0xbc99c06471dc6a3d),
    (0x3ff4f0000ff03ee2, 0x3c977f890b85531c),
    (0x3ff5100012dc4bd1, 0x3c6004657166a436),
    (0x3ff530001605277a, 0xbc96bfcece233209),
    (0x3ff54fffecdb704c, 0xbc8902720505a1d7),
    (0x3ff56fffef5f54a9, 0x3c9bbfe60ec96412),
    (0x3ff5900017e61012, 0x3c887ec581afef90),
    (0x3ff5b00003c93e92, 0xbc9f41080abf0cc0),
    (0x3ff5d0001d4919bc, 0xbc98812afb254729),
    (0x3ff5efffe7b87a89, 0xbc947eb780ed6904),
];

// ============================================================
// logf (single precision) data
// ============================================================

const LOGF_TABLE_BITS: u32 = 4;
const LOGF_N: usize = 1 << LOGF_TABLE_BITS;
const LOGF_LN2: f64 = asdouble(0x3fe62e42fefa39ef);

const LOGF_TAB: [(f64, f64); 16] = [
    (asdouble(0x3ff661ec79f8f3be), asdouble(0xbfd57bf7808caade)),
    (asdouble(0x3ff571ed4aaf883d), asdouble(0xbfd2bef0a7c06ddb)),
    (asdouble(0x3ff49539f0f010b0), asdouble(0xbfd01eae7f513a67)),
    (asdouble(0x3ff3c995b0b80385), asdouble(0xbfcb31d8a68224e9)),
    (asdouble(0x3ff30d190c8864a5), asdouble(0xbfc6574f0ac07758)),
    (asdouble(0x3ff25e227b0b8ea0), asdouble(0xbfc1aa2bc79c8100)),
    (asdouble(0x3ff1bb4a4a1a343f), asdouble(0xbfba4e76ce8c0e5e)),
    (asdouble(0x3ff12358f08ae5ba), asdouble(0xbfb1973c5a611ccc)),
    (asdouble(0x3ff0953f419900a7), asdouble(0xbfa252f438e10c1e)),
    (asdouble(0x3ff0000000000000), asdouble(0x0000000000000000)),
    (asdouble(0x3fee608cfd9a47ac), asdouble(0x3faaa5aa5df25984)),
    (asdouble(0x3feca4b31f026aa0), asdouble(0x3fbc5e53aa362eb4)),
    (asdouble(0x3feb2036576afce6), asdouble(0x3fc526e57720db08)),
    (asdouble(0x3fe9c2d163a1aa2d), asdouble(0x3fcbc2860d224770)),
    (asdouble(0x3fe886e6037841ed), asdouble(0x3fd1058bc8a07ee1)),
    (asdouble(0x3fe767dcf5534862), asdouble(0x3fd4043057b6ee09)),
];

const LOGF_POLY: [f64; 3] = [
    asdouble(0xbfd00ea348b88334),
    asdouble(0x3fd5575b0be00b6a),
    asdouble(0xbfdffffef20a4123),
];

// ============================================================
// log2 (double precision) data
// ============================================================

const LOG2_TABLE_BITS: u32 = 6;
const LOG2_N: usize = 1 << LOG2_TABLE_BITS;
const INVLN2HI: f64 = asdouble(0x3ff7154765200000);
const INVLN2LO: f64 = asdouble(0x3de705fc2eefa200);

const LOG2_POLY1: [f64; 10] = [
    asdouble(0xbfe71547652b82fe),
    asdouble(0x3fdec709dc3a03f7),
    asdouble(0xbfd71547652b7c3f),
    asdouble(0x3fd2776c50f05be4),
    asdouble(0xbfcec709dd768fe5),
    asdouble(0x3fca61761ec4e736),
    asdouble(0xbfc7153fbc64a79b),
    asdouble(0x3fc484d154f01b4a),
    asdouble(0xbfc289e4a72c383c),
    asdouble(0x3fc0b32f285aee66),
];

const LOG2_POLY: [f64; 6] = [
    asdouble(0xbfe71547652b8339),
    asdouble(0x3fdec709dc3a04be),
    asdouble(0xbfd7154764702ffb),
    asdouble(0x3fd2776c50034c48),
    asdouble(0xbfcec7b328ea92bc),
    asdouble(0x3fca6225e117f92e),
];

const LOG2_TAB: [(u64, u64); 64] = [
    (0x3ff724286bb1acf8, 0xbfe1095feecdb000),
    (0x3ff6e1f766d2cca1, 0xbfe08494bd76d000),
    (0x3ff6a13d0e30d48a, 0xbfe00143aee8f800),
    (0x3ff661ec32d06c85, 0xbfdefec5360b4000),
    (0x3ff623fa951198f8, 0xbfddfdd91ab7e000),
    (0x3ff5e75ba4cf026c, 0xbfdcffae0cc79000),
    (0x3ff5ac055a214fb8, 0xbfdc043811fda000),
    (0x3ff571ed0f166e1e, 0xbfdb0b67323ae000),
    (0x3ff53909590bf835, 0xbfda152f5a2db000),
    (0x3ff5014fed61addd, 0xbfd9217f5af86000),
    (0x3ff4cab88e487bd0, 0xbfd8304db0719000),
    (0x3ff49539b4334fee, 0xbfd74189f9a9e000),
    (0x3ff460cbdfafd569, 0xbfd6552bb5199000),
    (0x3ff42d664ee4b953, 0xbfd56b23a29b1000),
    (0x3ff3fb01111dd8a6, 0xbfd483650f5fa000),
    (0x3ff3c995b70c5836, 0xbfd39de937f6a000),
    (0x3ff3991c4ab6fd4a, 0xbfd2baa1538d6000),
    (0x3ff3698e0ce099b5, 0xbfd1d98340ca4000),
    (0x3ff33ae48213e7b2, 0xbfd0fa853a40e000),
    (0x3ff30d191985bdb1, 0xbfd01d9c32e73000),
    (0x3ff2e025cab271d7, 0xbfce857da2fa6000),
    (0x3ff2b404cf13cd82, 0xbfccd3c8633d8000),
    (0x3ff288b02c7ccb50, 0xbfcb26034c14a000),
    (0x3ff25e2263944de5, 0xbfc97c1c2f4fe000),
    (0x3ff234563d8615b1, 0xbfc7d6023f800000),
    (0x3ff20b46e33eaf38, 0xbfc633a71a05e000),
    (0x3ff1e2eefdcda3dd, 0xbfc494f5e9570000),
    (0x3ff1bb4a580b3930, 0xbfc2f9e424e0a000),
    (0x3ff19453847f2200, 0xbfc162595afdc000),
    (0x3ff16e06c0d5d73c, 0xbfbf9c9a75bd8000),
    (0x3ff1485f47b7e4c2, 0xbfbc7b575bf9c000),
    (0x3ff12358ad0085d1, 0xbfb960c60ff48000),
    (0x3ff0fef00f532227, 0xbfb64ce247b60000),
    (0x3ff0db2077d03a8f, 0xbfb33f78b2014000),
    (0x3ff0b7e6d65980d9, 0xbfb0387d1a42c000),
    (0x3ff0953efe7b408d, 0xbfaa6f9208b50000),
    (0x3ff07325cac53b83, 0xbfa47a954f770000),
    (0x3ff05197e40d1b5c, 0xbf9d23a8c50c0000),
    (0x3ff03091c1208ea2, 0xbf916a2629780000),
    (0x3ff0101025b37e21, 0xbf7720f8d8e80000),
    (0x3fefc07ef9caa76b, 0x3f86fe53b1500000),
    (0x3fef4465d3f6f184, 0x3fa11ccce10f8000),
    (0x3feecc079f84107f, 0x3fac4dfc8c8b8000),
    (0x3fee573a99975ae8, 0x3fb3aa321e574000),
    (0x3fede5d6f0bd3de6, 0x3fb918a0d08b8000),
    (0x3fed77b681ff38b3, 0x3fbe72e9da044000),
    (0x3fed0cb5724de943, 0x3fc1dcd2507f6000),
    (0x3feca4b2dc0e7563, 0x3fc476ab03dea000),
    (0x3fec3f8ee8d6cb51, 0x3fc7074377e22000),
    (0x3febdd2b4f020c4c, 0x3fc98ede8ba94000),
    (0x3feb7d6c006015ca, 0x3fcc0db86ad2e000),
    (0x3feb20366e2e338f, 0x3fce840aafcee000),
    (0x3feac57026295039, 0x3fd0790ab4678000),
    (0x3fea6d01bc2731dd, 0x3fd1ac056801c000),
    (0x3fea16d3bc3ff18b, 0x3fd2db11d4fee000),
    (0x3fe9c2d14967fead, 0x3fd406464ec58000),
    (0x3fe970e4f47c9902, 0x3fd52dbe093af000),
    (0x3fe920fb3982bcf2, 0x3fd651902050d000),
    (0x3fe8d30187f759f1, 0x3fd771d2cdeaf000),
    (0x3fe886e5ebb9f66d, 0x3fd88e9c857d9000),
    (0x3fe83c97b658b994, 0x3fd9a80155e16000),
    (0x3fe7f405ffc61022, 0x3fdabe186ed3d000),
    (0x3fe7ad22181415ca, 0x3fdbd0f2aea0e000),
    (0x3fe767dcf99eff8c, 0x3fdce0a43dbf4000),
];

const LOG2_TAB2: [(u64, u64); 64] = [
    (0x3fe6200012b90a8e, 0x3c8904ab0644b605),
    (0x3fe66000045734a6, 0x3c61ff9bea62f7a9),
    (0x3fe69fffc325f2c5, 0x3c827ecfcb3c90ba),
    (0x3fe6e00038b95a04, 0x3c88ff8856739326),
    (0x3fe71fffe09994e3, 0x3c8afd40275f82b1),
    (0x3fe7600015590e10, 0xbc72fd75b4238341),
    (0x3fe7a00012655bd5, 0x3c7808e67c242b76),
    (0x3fe7e0003259e9a6, 0xbc6208e426f622b7),
    (0x3fe81fffedb4b2d2, 0xbc8402461ea5c92f),
    (0x3fe860002dfafcc3, 0x3c6df7f4a2f29a1f),
    (0x3fe89ffff78c6b50, 0xbc8e0453094995fd),
    (0x3fe8e00039671566, 0xbc8a04f3bec77b45),
    (0x3fe91fffe2bf1745, 0xbc77fa34400e203c),
    (0x3fe95fffcc5c9fd1, 0xbc76ff8005a0695d),
    (0x3fe9a0003bba4767, 0x3c70f8c4c4ec7e03),
    (0x3fe9dfffe7b92da5, 0x3c8e7fd9478c4602),
    (0x3fea1fffd72efdaf, 0xbc6a0c554dcdae7e),
    (0x3fea5fffde04ff95, 0x3c867da98ce9b26b),
    (0x3fea9fffca5e8d2b, 0xbc8284c9b54c13de),
    (0x3feadfffddad03ea, 0x3c5812c8ea602e3c),
    (0x3feb1ffff10d3d4d, 0xbc8efaddad27789c),
    (0x3feb5fffce21165a, 0x3c53cb1719c61237),
    (0x3feb9fffd950e674, 0x3c73f7d94194ce00),
    (0x3febe000139ca8af, 0x3c750ac4215d9bc0),
    (0x3fec20005b46df99, 0x3c6beea653e9c1c9),
    (0x3fec600040b9f7ae, 0xbc7c079f274a70d6),
    (0x3feca0006255fd8a, 0xbc7a0b4076e84c1f),
    (0x3fecdfffd94c095d, 0x3c88f933f99ab5d7),
    (0x3fed1ffff975d6cf, 0xbc582c08665fe1be),
    (0x3fed5fffa2561c93, 0xbc7b04289bd295f3),
    (0x3fed9fff9d228b0c, 0x3c870251340fa236),
    (0x3fede00065bc7e16, 0xbc75011e16a4d80c),
    (0x3fee200002f64791, 0x3c89802f09ef62e0),
    (0x3fee600057d7a6d8, 0xbc7e0b75580cf7fa),
    (0x3feea00027edc00c, 0xbc8c848309459811),
    (0x3feee0006cf5cb7c, 0xbc8f8027951576f4),
    (0x3fef2000782b7dcc, 0xbc8f81d97274538f),
    (0x3fef6000260c450a, 0xbc4071002727ffdc),
    (0x3fef9fffe88cd533, 0xbc581bdce1fda8b0),
    (0x3fefdfffd50f8689, 0x3c87f91acb918e6e),
    (0x3ff0200004292367, 0x3c9b7ff365324681),
    (0x3ff05fffe3e3d668, 0x3c86fa08ddae957b),
    (0x3ff0a0000a85a757, 0xbc57e2de80d3fb91),
    (0x3ff0e0001a5f3fcc, 0xbc91823305c5f014),
    (0x3ff11ffff8afbaf5, 0xbc8bfabb6680bac2),
    (0x3ff15fffe54d91ad, 0xbc9d7f121737e7ef),
    (0x3ff1a00011ac36e1, 0x3c9c000a0516f5ff),
    (0x3ff1e00019c84248, 0xbc9082fbe4da5da0),
    (0x3ff220000ffe5e6e, 0xbc88fdd04c9cfb43),
    (0x3ff26000269fd891, 0x3c8cfe2a7994d182),
    (0x3ff2a00029a6e6da, 0xbc700273715e8bc5),
    (0x3ff2dfffe0293e39, 0x3c9b7c39dab2a6f9),
    (0x3ff31ffff7dcf082, 0x3c7df1336edc5254),
    (0x3ff35ffff05a8b60, 0xbc9e03564ccd31eb),
    (0x3ff3a0002e0eaecc, 0x3c75f0e74bd3a477),
    (0x3ff3e000043bb236, 0x3c9c7dcb149d8833),
    (0x3ff4200002d187ff, 0x3c7e08afcf2d3d28),
    (0x3ff460000d387cb1, 0x3c820837856599a6),
    (0x3ff4a00004569f89, 0xbc89fa5c904fbcd2),
    (0x3ff4e000043543f3, 0xbc781125ed175329),
    (0x3ff51fffcc027f0f, 0x3c9883d8847754dc),
    (0x3ff55ffffd87b36f, 0xbc8709e731d02807),
    (0x3ff59ffff21df7ba, 0x3c87f79f68727b02),
    (0x3ff5dfffebfc3481, 0xbc9180902e30e93e),
];

// ============================================================
// log2f (single precision) data
// ============================================================

const LOG2F_TABLE_BITS: u32 = 4;
const LOG2F_N: usize = 1 << LOG2F_TABLE_BITS;

const LOG2F_TAB: [(f64, f64); 16] = [
    (asdouble(0x3ff661ec79f8f3be), asdouble(0xbfdefec65b963019)),
    (asdouble(0x3ff571ed4aaf883d), asdouble(0xbfdb0b6832d4fca4)),
    (asdouble(0x3ff49539f0f010b0), asdouble(0xbfd7418b0a1fb77b)),
    (asdouble(0x3ff3c995b0b80385), asdouble(0xbfd39de91a6dcf7b)),
    (asdouble(0x3ff30d190c8864a5), asdouble(0xbfd01d9bf3f2b631)),
    (asdouble(0x3ff25e227b0b8ea0), asdouble(0xbfc97c1d1b3b7af0)),
    (asdouble(0x3ff1bb4a4a1a343f), asdouble(0xbfc2f9e393af3c9f)),
    (asdouble(0x3ff12358f08ae5ba), asdouble(0xbfb960cbbf788d5c)),
    (asdouble(0x3ff0953f419900a7), asdouble(0xbfaa6f9db6475fce)),
    (asdouble(0x3ff0000000000000), asdouble(0x0000000000000000)),
    (asdouble(0x3fee608cfd9a47ac), asdouble(0x3fb338ca9f24f53d)),
    (asdouble(0x3feca4b31f026aa0), asdouble(0x3fc476a9543891ba)),
    (asdouble(0x3feb2036576afce6), asdouble(0x3fce840b4ac4e4d2)),
    (asdouble(0x3fe9c2d163a1aa2d), asdouble(0x3fd40645f0c6651c)),
    (asdouble(0x3fe886e6037841ed), asdouble(0x3fd88e9c2c1b9ff8)),
    (asdouble(0x3fe767dcf5534862), asdouble(0x3fdce0a44eb17bcc)),
];

const LOG2F_POLY: [f64; 4] = [
    asdouble(0xbfd712b6f70a7e4d),
    asdouble(0x3fdecabf496832e0),
    asdouble(0xbfe715479ffae3de),
    asdouble(0x3ff715475f35c8b8),
];

// ============================================================
// log10 / log10f scalar constants
// ============================================================

const IVLN10HI: f64 = asdouble(0x3fdbcb7b15200000);
const IVLN10LO: f64 = asdouble(0x3dbb9438ca9aadd5);
const LOG10_2HI: f64 = asdouble(0x3fd34413509f6000);
const LOG10_2LO: f64 = asdouble(0x3d59fef311f12b36);
const LG1: f64 = asdouble(0x3fe5555555555593);
const LG2: f64 = asdouble(0x3fd999999997fa04);
const LG3: f64 = asdouble(0x3fd2492494229359);
const LG4: f64 = asdouble(0x3fcc71c51d8e78af);
const LG5: f64 = asdouble(0x3fc7466496cb03de);
const LG6: f64 = asdouble(0x3fc39a09d078c69f);
const LG7: f64 = asdouble(0x3fc2f112df3e5244);

const IVLN10HI_F: f32 = asfloat(0x3ede6000);
const IVLN10LO_F: f32 = asfloat(0xb804ead9);
const LOG10_2HI_F: f32 = asfloat(0x3e9a2080);
const LOG10_2LO_F: f32 = asfloat(0x355427db);
const LG1_F: f32 = asfloat(0x3f2aaaaa);
const LG2_F: f32 = asfloat(0x3eccce13);
const LG3_F: f32 = asfloat(0x3e91e9ee);
const LG4_F: f32 = asfloat(0x3e789e26);

// ============================================================
// Helper
// ============================================================

#[inline]
fn log_top16(x: f64) -> u32 {
    (asuint64(x) >> 48) as u32
}

// ============================================================
// log (double precision)
// ============================================================

const LOG_LO: u64 = asuint64(asdouble(0x3fee000000000000));
const LOG_HI: u64 = asuint64(asdouble(0x3ff1090000000000));

#[no_mangle]
pub extern "C" fn log(x: f64) -> f64 {
    let ix = asuint64(x);
    let top = log_top16(x);

    if predict_false(ix.wrapping_sub(LOG_LO) < LOG_HI.wrapping_sub(LOG_LO)) {
        if predict_false(ix == asuint64(1.0)) {
            return 0.0;
        }
        let r = x - 1.0;
        let r2 = r * r;
        let r3 = r * r2;
        let y = r3
            * (LOG_POLY1[1]
                + r * LOG_POLY1[2]
                + r2 * LOG_POLY1[3]
                + r3
                    * (LOG_POLY1[4]
                        + r * LOG_POLY1[5]
                        + r2 * LOG_POLY1[6]
                        + r3
                            * (LOG_POLY1[7]
                                + r * LOG_POLY1[8]
                                + r2 * LOG_POLY1[9]
                                + r3 * LOG_POLY1[10])));
        let w = r * 134217728.0;
        let rhi = r + w - w;
        let rlo = r - rhi;
        let w = rhi * rhi * LOG_POLY1[0];
        let hi = r + w;
        let lo = r - hi + w;
        let lo = lo + LOG_POLY1[0] * rlo * (rhi + r);
        let y = y + lo;
        let y = y + hi;
        return eval_as_double(y);
    }

    if predict_false(top.wrapping_sub(0x0010) >= 0x7ff0 - 0x0010) {
        if ix * 2 == 0 {
            return __math_divzero(1);
        }
        if ix == asuint64(f64::INFINITY) {
            return x;
        }
        if (top & 0x8000) != 0 || (top & 0x7ff0) == 0x7ff0 {
            return __math_invalid(x);
        }
        let ix = asuint64(x * 4503599627370496.0);
        let ix = ix - (52u64 << 52);
        return log_main(ix);
    }

    log_main(ix)
}

#[inline]
fn log_main(ix: u64) -> f64 {
    const OFF: u64 = 0x3fe6000000000000;

    let tmp = ix - OFF;
    let i = ((tmp >> (52 - LOG_TABLE_BITS)) % LOG_N as u64) as usize;
    let k = (tmp as i64 >> 52) as i32;
    let iz = ix - (tmp & (0xfffu64 << 52));
    let invc = asdouble(LOG_TAB[i].0);
    let logc = asdouble(LOG_TAB[i].1);
    let z = asdouble(iz);

    let r = (z - asdouble(LOG_TAB2[i].0) - asdouble(LOG_TAB2[i].1)) * invc;
    let kd = k as f64;

    let w = kd * LN2HI + logc;
    let hi = w + r;
    let lo = w - hi + r + kd * LN2LO;

    let r2 = r * r;
    let y = lo
        + r2 * LOG_POLY[0]
        + r * r2 * (LOG_POLY[1] + r * LOG_POLY[2] + r2 * (LOG_POLY[3] + r * LOG_POLY[4]))
        + hi;
    eval_as_double(y)
}

// ============================================================
// logf (single precision)
// ============================================================

#[no_mangle]
pub extern "C" fn logf(x: f32) -> f32 {
    let ix = asuint(x);

    if predict_false(ix == 0x3f800000) {
        return 0.0;
    }
    if predict_false(ix.wrapping_sub(0x00800000) >= 0x7f800000 - 0x00800000) {
        if ix * 2 == 0 {
            return __math_divzerof(1);
        }
        if ix == 0x7f800000 {
            return x;
        }
        if (ix & 0x80000000) != 0 || ix * 2 >= 0xff000000 {
            return __math_invalidf(x);
        }
        let ix = asuint(x * 8388608.0f32);
        let ix = ix - (23u32 << 23);
        return logf_main(ix);
    }

    logf_main(ix)
}

#[inline]
fn logf_main(ix: u32) -> f32 {
    const OFF: u32 = 0x3f330000;

    let tmp = ix - OFF;
    let i = ((tmp >> (23 - LOGF_TABLE_BITS)) % LOGF_N as u32) as usize;
    let k = (tmp as i32 >> 23) as i32;
    let iz = ix - (tmp & 0xff800000);
    let invc = LOGF_TAB[i].0;
    let logc = LOGF_TAB[i].1;
    let z = asfloat(iz) as f64;

    let r = z * invc - 1.0;
    let y0 = logc + k as f64 * LOGF_LN2;

    let r2 = r * r;
    let y = LOGF_POLY[1] * r + LOGF_POLY[2];
    let y = LOGF_POLY[0] * r2 + y;
    let y = y * r2 + (y0 + r);
    eval_as_float(y as f32)
}

// ============================================================
// log2 (double precision)
// ============================================================

const LOG2_LO: u64 = asuint64(asdouble(0x3feea4af00000000));
const LOG2_HI: u64 = asuint64(asdouble(0x3ff0b55900000000));

#[no_mangle]
pub extern "C" fn log2(x: f64) -> f64 {
    let ix = asuint64(x);
    let top = log_top16(x);

    if predict_false(ix.wrapping_sub(LOG2_LO) < LOG2_HI.wrapping_sub(LOG2_LO)) {
        if predict_false(ix == asuint64(1.0)) {
            return 0.0;
        }
        let r = x - 1.0;
        let rhi = asdouble(asuint64(r) & (0xFFFFFFFFu64 << 32));
        let rlo = r - rhi;
        let hi = rhi * INVLN2HI;
        let lo = rlo * INVLN2HI + r * INVLN2LO;
        let r2 = r * r;
        let r4 = r2 * r2;
        let p = r2 * (LOG2_POLY1[0] + r * LOG2_POLY1[1]);
        let y = hi + p;
        let lo = lo + hi - y + p;
        let lo = lo
            + r4
                * (LOG2_POLY1[2]
                    + r * LOG2_POLY1[3]
                    + r2 * (LOG2_POLY1[4] + r * LOG2_POLY1[5])
                    + r4
                        * (LOG2_POLY1[6]
                            + r * LOG2_POLY1[7]
                            + r2 * (LOG2_POLY1[8] + r * LOG2_POLY1[9])));
        let y = y + lo;
        return eval_as_double(y);
    }

    if predict_false(top.wrapping_sub(0x0010) >= 0x7ff0 - 0x0010) {
        if ix * 2 == 0 {
            return __math_divzero(1);
        }
        if ix == asuint64(f64::INFINITY) {
            return x;
        }
        if (top & 0x8000) != 0 || (top & 0x7ff0) == 0x7ff0 {
            return __math_invalid(x);
        }
        let ix = asuint64(x * 4503599627370496.0);
        let ix = ix - (52u64 << 52);
        return log2_main(ix);
    }

    log2_main(ix)
}

#[inline]
fn log2_main(ix: u64) -> f64 {
    const OFF: u64 = 0x3fe6000000000000;

    let tmp = ix - OFF;
    let i = ((tmp >> (52 - LOG2_TABLE_BITS)) % LOG2_N as u64) as usize;
    let k = (tmp as i64 >> 52) as i32;
    let iz = ix - (tmp & (0xfffu64 << 52));
    let invc = asdouble(LOG2_TAB[i].0);
    let logc = asdouble(LOG2_TAB[i].1);
    let z = asdouble(iz);
    let kd = k as f64;

    let r = (z - asdouble(LOG2_TAB2[i].0) - asdouble(LOG2_TAB2[i].1)) * invc;
    let rhi = asdouble(asuint64(r) & (0xFFFFFFFFu64 << 32));
    let rlo = r - rhi;
    let t1 = rhi * INVLN2HI;
    let t2 = rlo * INVLN2HI + r * INVLN2LO;

    let t3 = kd + logc;
    let hi = t3 + t1;
    let lo = t3 - hi + t1 + t2;

    let r2 = r * r;
    let r4 = r2 * r2;
    let p = LOG2_POLY[0]
        + r * LOG2_POLY[1]
        + r2 * (LOG2_POLY[2] + r * LOG2_POLY[3])
        + r4 * (LOG2_POLY[4] + r * LOG2_POLY[5]);
    let y = lo + r2 * p + hi;
    eval_as_double(y)
}

// ============================================================
// log2f (single precision)
// ============================================================

#[no_mangle]
pub extern "C" fn log2f(x: f32) -> f32 {
    let ix = asuint(x);

    if predict_false(ix == 0x3f800000) {
        return 0.0;
    }
    if predict_false(ix.wrapping_sub(0x00800000) >= 0x7f800000 - 0x00800000) {
        if ix * 2 == 0 {
            return __math_divzerof(1);
        }
        if ix == 0x7f800000 {
            return x;
        }
        if (ix & 0x80000000) != 0 || ix * 2 >= 0xff000000 {
            return __math_invalidf(x);
        }
        let ix = asuint(x * 8388608.0f32);
        let ix = ix - (23u32 << 23);
        return log2f_main(ix);
    }

    log2f_main(ix)
}

#[inline]
fn log2f_main(ix: u32) -> f32 {
    const OFF: u32 = 0x3f330000;

    let tmp = ix - OFF;
    let i = ((tmp >> (23 - LOG2F_TABLE_BITS)) % LOG2F_N as u32) as usize;
    let top = tmp & 0xff800000;
    let iz = ix - top;
    let k = (tmp as i32 >> 23) as i32;
    let invc = LOG2F_TAB[i].0;
    let logc = LOG2F_TAB[i].1;
    let z = asfloat(iz) as f64;

    let r = z * invc - 1.0;
    let y0 = logc + k as f64;

    let r2 = r * r;
    let y = LOG2F_POLY[1] * r + LOG2F_POLY[2];
    let y = LOG2F_POLY[0] * r2 + y;
    let p = LOG2F_POLY[3] * r + y0;
    let y = y * r2 + p;
    eval_as_float(y as f32)
}

// ============================================================
// log10 (double precision)
// ============================================================

#[no_mangle]
pub extern "C" fn log10(x: f64) -> f64 {
    let mut hx = (asuint64(x) >> 32) as u32;
    let mut k: i32 = 0;
    let mut x = x;

    if hx < 0x00100000 || (hx >> 31) != 0 {
        if asuint64(x) << 1 == 0 {
            return -1.0 / (x * x);
        }
        if (hx >> 31) != 0 {
            return (x - x) / 0.0;
        }
        k -= 54;
        x *= 18014398509481984.0;
        hx = (asuint64(x) >> 32) as u32;
    } else if hx >= 0x7ff00000 {
        return x;
    } else if hx == 0x3ff00000 && (asuint64(x) << 32) == 0 {
        return 0.0;
    }

    hx = hx.wrapping_add(0x3ff00000 - 0x3fe6a09e);
    k += (hx >> 20) as i32 - 0x3ff;
    hx = (hx & 0x000fffff) + 0x3fe6a09e;
    x = asdouble(((hx as u64) << 32) | (asuint64(x) & 0xffffffff));

    let f = x - 1.0;
    let hfsq = 0.5 * f * f;
    let s = f / (2.0 + f);
    let z = s * s;
    let w = z * z;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    let r = t2 + t1;

    let hi = f - hfsq;
    let hi = asdouble(asuint64(hi) & (0xFFFFFFFFu64 << 32));
    let lo = f - hi - hfsq + s * (hfsq + r);

    let val_hi = hi * IVLN10HI;
    let dk = k as f64;
    let y = dk * LOG10_2HI;
    let val_lo = dk * LOG10_2LO + (lo + hi) * IVLN10LO + lo * IVLN10HI;

    let w = y + val_hi;
    let val_lo = val_lo + (y - w) + val_hi;
    let val_hi = w;

    val_lo + val_hi
}

// ============================================================
// log10f (single precision)
// ============================================================

#[no_mangle]
pub extern "C" fn log10f(x: f32) -> f32 {
    let mut ix = asuint(x);
    let mut k: i32 = 0;
    let mut x = x;

    if ix < 0x00800000 || (ix >> 31) != 0 {
        if ix << 1 == 0 {
            return -1.0 / (x * x);
        }
        if (ix >> 31) != 0 {
            return (x - x) / 0.0f32;
        }
        k -= 25;
        x *= 33554432.0f32;
        ix = asuint(x);
    } else if ix >= 0x7f800000 {
        return x;
    } else if ix == 0x3f800000 {
        return 0.0;
    }

    ix = ix.wrapping_add(0x3f800000 - 0x3f3504f3);
    k += (ix >> 23) as i32 - 0x7f;
    ix = (ix & 0x007fffff) + 0x3f3504f3;
    x = asfloat(ix);

    let f = x - 1.0f32;
    let s = f / (2.0f32 + f);
    let z = s * s;
    let w = z * z;
    let t1 = w * (LG2_F + w * LG4_F);
    let t2 = z * (LG1_F + w * LG3_F);
    let r = t2 + t1;
    let hfsq = 0.5f32 * f * f;

    let hi = f - hfsq;
    let hi = asfloat(asuint(hi) & 0xfffff000);
    let lo = f - hi - hfsq + s * (hfsq + r);
    let dk = k as f32;

    dk * LOG10_2LO_F + (lo + hi) * IVLN10LO_F + lo * IVLN10HI_F + hi * IVLN10HI_F + dk * LOG10_2HI_F
}
