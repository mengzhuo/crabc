// Translated from musl: pow.c, powf.c, pow_data.c, powf_data.c
// Uses non-FMA paths throughout (no __FP_FAST_FMA).
// issignaling_inline/issignalingf_inline defined as 0 (always false) per musl libm.h.

// ============================================================
// pow (double precision) data
// ============================================================

const POW_LOG_TABLE_BITS: u32 = 7;
const POW_N: u64 = 1 << POW_LOG_TABLE_BITS;
const POW_OFF: u64 = 0x3FE6955500000000;

const POW_LN2HI: f64 = asdouble(0x3FE62E42FEFA3800);
const POW_LN2LO: f64 = asdouble(0x3D2EF35793C76730);

// Polynomial coefficients for log1p(r) - r - A[0]*r^2.
// First coefficient A[0] = -0.5 is implicit.
const POW_POLY: [f64; 7] = [
    asdouble(0xBFE0000000000000), // A[0] = -0.5
    asdouble(0xBFE5555555555560), // A[1]
    asdouble(0x3FE0000000000006), // A[2]
    asdouble(0x3FE999999959554E), // A[3]
    asdouble(0xBFE555555529A47A), // A[4]
    asdouble(0xBFF2495B9B4845E9), // A[5]
    asdouble(0x3FF0002B8B263FC3), // A[6]
];

// Table entries: (invc, logc, logctail)
const POW_LOG_TAB: [(f64, f64, f64); 128] = [
    (asdouble(0x3FF6A00000000000), asdouble(0xBFD62C82F2B9C800), asdouble(0x3CFAB42428375680)),
    (asdouble(0x3FF6800000000000), asdouble(0xBFD5D1BDBF580800), asdouble(0xBD1CA508D8E0F720)),
    (asdouble(0x3FF6600000000000), asdouble(0xBFD5767717455800), asdouble(0xBD2362A4D5B6506D)),
    (asdouble(0x3FF6400000000000), asdouble(0xBFD51AAD872DF800), asdouble(0xBCE684E49EB067D5)),
    (asdouble(0x3FF6200000000000), asdouble(0xBFD4BE5F95777800), asdouble(0xBD041B6993293EE0)),
    (asdouble(0x3FF6000000000000), asdouble(0xBFD4618BC21C6000), asdouble(0x3D13D82F484C84CC)),
    (asdouble(0x3FF5E00000000000), asdouble(0xBFD404308686A800), asdouble(0x3CDC42F3ED820B3A)),
    (asdouble(0x3FF5C00000000000), asdouble(0xBFD3A64C55694800), asdouble(0x3D20B1C686519460)),
    (asdouble(0x3FF5A00000000000), asdouble(0xBFD347DD9A988000), asdouble(0x3D25594DD4C58092)),
    (asdouble(0x3FF5800000000000), asdouble(0xBFD2E8E2BAE12000), asdouble(0x3D267B1E99B72BD8)),
    (asdouble(0x3FF5600000000000), asdouble(0xBFD2895A13DE8800), asdouble(0x3D15CA14B6CFB03F)),
    (asdouble(0x3FF5600000000000), asdouble(0xBFD2895A13DE8800), asdouble(0x3D15CA14B6CFB03F)),
    (asdouble(0x3FF5400000000000), asdouble(0xBFD22941FBCF7800), asdouble(0xBD165A242853DA76)),
    (asdouble(0x3FF5200000000000), asdouble(0xBFD1C898C1699800), asdouble(0xBD1FAFBC68E75404)),
    (asdouble(0x3FF5000000000000), asdouble(0xBFD1675CABABA800), asdouble(0x3D1F1FC63382A8F0)),
    (asdouble(0x3FF4E00000000000), asdouble(0xBFD1058BF9AE4800), asdouble(0xBD26A8C4FD055A66)),
    (asdouble(0x3FF4C00000000000), asdouble(0xBFD0A324E2739000), asdouble(0xBD0C6BEE7EF4030E)),
    (asdouble(0x3FF4A00000000000), asdouble(0xBFD0402594B4D000), asdouble(0xBCF036B89EF42D7F)),
    (asdouble(0x3FF4A00000000000), asdouble(0xBFD0402594B4D000), asdouble(0xBCF036B89EF42D7F)),
    (asdouble(0x3FF4800000000000), asdouble(0xBFCFB9186D5E4000), asdouble(0x3D0D572AAB993C87)),
    (asdouble(0x3FF4600000000000), asdouble(0xBFCEF0ADCBDC6000), asdouble(0x3D2B26B79C86AF24)),
    (asdouble(0x3FF4400000000000), asdouble(0xBFCE27076E2AF000), asdouble(0xBD172F4F543FFF10)),
    (asdouble(0x3FF4200000000000), asdouble(0xBFCD5C216B4FC000), asdouble(0x3D21BA91BBCA681B)),
    (asdouble(0x3FF4000000000000), asdouble(0xBFCC8FF7C79AA000), asdouble(0x3D27794F689F8434)),
    (asdouble(0x3FF4000000000000), asdouble(0xBFCC8FF7C79AA000), asdouble(0x3D27794F689F8434)),
    (asdouble(0x3FF3E00000000000), asdouble(0xBFCBC286742D9000), asdouble(0x3D194EB0318BB78F)),
    (asdouble(0x3FF3C00000000000), asdouble(0xBFCAF3C94E80C000), asdouble(0x3CBA4E633FCD9066)),
    (asdouble(0x3FF3A00000000000), asdouble(0xBFCA23BC1FE2B000), asdouble(0xBD258C64DC46C1EA)),
    (asdouble(0x3FF3A00000000000), asdouble(0xBFCA23BC1FE2B000), asdouble(0xBD258C64DC46C1EA)),
    (asdouble(0x3FF3800000000000), asdouble(0xBFC9525A9CF45000), asdouble(0xBD2AD1D904C1D4E3)),
    (asdouble(0x3FF3600000000000), asdouble(0xBFC87FA06520D000), asdouble(0x3D2BBDBF7FDBFA09)),
    (asdouble(0x3FF3400000000000), asdouble(0xBFC7AB890210E000), asdouble(0x3D2BDB9072534A58)),
    (asdouble(0x3FF3400000000000), asdouble(0xBFC7AB890210E000), asdouble(0x3D2BDB9072534A58)),
    (asdouble(0x3FF3200000000000), asdouble(0xBFC6D60FE719D000), asdouble(0xBD10E46AA3B2E266)),
    (asdouble(0x3FF3000000000000), asdouble(0xBFC5FF3070A79000), asdouble(0xBD1E9E439F105039)),
    (asdouble(0x3FF3000000000000), asdouble(0xBFC5FF3070A79000), asdouble(0xBD1E9E439F105039)),
    (asdouble(0x3FF2E00000000000), asdouble(0xBFC526E5E3A1B000), asdouble(0xBD20DE8B90075B8F)),
    (asdouble(0x3FF2C00000000000), asdouble(0xBFC44D2B6CCB8000), asdouble(0x3D170CC16135783C)),
    (asdouble(0x3FF2C00000000000), asdouble(0xBFC44D2B6CCB8000), asdouble(0x3D170CC16135783C)),
    (asdouble(0x3FF2A00000000000), asdouble(0xBFC371FC201E9000), asdouble(0x3CF178864D27543A)),
    (asdouble(0x3FF2800000000000), asdouble(0xBFC29552F81FF000), asdouble(0xBD248D301771C408)),
    (asdouble(0x3FF2600000000000), asdouble(0xBFC1B72AD52F6000), asdouble(0xBD2E80A41811A396)),
    (asdouble(0x3FF2600000000000), asdouble(0xBFC1B72AD52F6000), asdouble(0xBD2E80A41811A396)),
    (asdouble(0x3FF2400000000000), asdouble(0xBFC0D77E7CD09000), asdouble(0x3D0A699688E85BF4)),
    (asdouble(0x3FF2400000000000), asdouble(0xBFC0D77E7CD09000), asdouble(0x3D0A699688E85BF4)),
    (asdouble(0x3FF2200000000000), asdouble(0xBFBFEC9131DBE000), asdouble(0xBD2575545CA333F2)),
    (asdouble(0x3FF2000000000000), asdouble(0xBFBE27076E2B0000), asdouble(0x3D2A342C2AF0003C)),
    (asdouble(0x3FF2000000000000), asdouble(0xBFBE27076E2B0000), asdouble(0x3D2A342C2AF0003C)),
    (asdouble(0x3FF1E00000000000), asdouble(0xBFBC5E548F5BC000), asdouble(0xBD1D0C57585FBE06)),
    (asdouble(0x3FF1C00000000000), asdouble(0xBFBA926D3A4AE000), asdouble(0x3D253935E85BAAC8)),
    (asdouble(0x3FF1C00000000000), asdouble(0xBFBA926D3A4AE000), asdouble(0x3D253935E85BAAC8)),
    (asdouble(0x3FF1A00000000000), asdouble(0xBFB8C345D631A000), asdouble(0x3D137C294D2F5668)),
    (asdouble(0x3FF1A00000000000), asdouble(0xBFB8C345D631A000), asdouble(0x3D137C294D2F5668)),
    (asdouble(0x3FF1800000000000), asdouble(0xBFB6F0D28AE56000), asdouble(0xBD269737C93373DA)),
    (asdouble(0x3FF1600000000000), asdouble(0xBFB51B073F062000), asdouble(0x3D1F025B61C65E57)),
    (asdouble(0x3FF1600000000000), asdouble(0xBFB51B073F062000), asdouble(0x3D1F025B61C65E57)),
    (asdouble(0x3FF1400000000000), asdouble(0xBFB341D7961BE000), asdouble(0x3D2C5EDACCF913DF)),
    (asdouble(0x3FF1400000000000), asdouble(0xBFB341D7961BE000), asdouble(0x3D2C5EDACCF913DF)),
    (asdouble(0x3FF1200000000000), asdouble(0xBFB16536EEA38000), asdouble(0x3D147C5E768FA309)),
    (asdouble(0x3FF1000000000000), asdouble(0xBFAF0A30C0118000), asdouble(0x3D2D599E83368E91)),
    (asdouble(0x3FF1000000000000), asdouble(0xBFAF0A30C0118000), asdouble(0x3D2D599E83368E91)),
    (asdouble(0x3FF0E00000000000), asdouble(0xBFAB42DD71198000), asdouble(0x3D1C827AE5D6704C)),
    (asdouble(0x3FF0E00000000000), asdouble(0xBFAB42DD71198000), asdouble(0x3D1C827AE5D6704C)),
    (asdouble(0x3FF0C00000000000), asdouble(0xBFA77458F632C000), asdouble(0xBD2CFC4634F2A1EE)),
    (asdouble(0x3FF0C00000000000), asdouble(0xBFA77458F632C000), asdouble(0xBD2CFC4634F2A1EE)),
    (asdouble(0x3FF0A00000000000), asdouble(0xBFA39E87B9FEC000), asdouble(0x3CF502B7F526FEAA)),
    (asdouble(0x3FF0A00000000000), asdouble(0xBFA39E87B9FEC000), asdouble(0x3CF502B7F526FEAA)),
    (asdouble(0x3FF0800000000000), asdouble(0xBF9F829B0E780000), asdouble(0xBD2980267C7E09E4)),
    (asdouble(0x3FF0800000000000), asdouble(0xBF9F829B0E780000), asdouble(0xBD2980267C7E09E4)),
    (asdouble(0x3FF0600000000000), asdouble(0xBF97B91B07D58000), asdouble(0xBD288D5493FAA639)),
    (asdouble(0x3FF0400000000000), asdouble(0xBF8FC0A8B0FC0000), asdouble(0xBCDF1E7CF6D3A69C)),
    (asdouble(0x3FF0400000000000), asdouble(0xBF8FC0A8B0FC0000), asdouble(0xBCDF1E7CF6D3A69C)),
    (asdouble(0x3FF0200000000000), asdouble(0xBF7FE02A6B100000), asdouble(0xBD19E23F0DDA40E4)),
    (asdouble(0x3FF0200000000000), asdouble(0xBF7FE02A6B100000), asdouble(0xBD19E23F0DDA40E4)),
    (asdouble(0x3FF0000000000000), asdouble(0x0000000000000000), asdouble(0x0000000000000000)),
    (asdouble(0x3FF0000000000000), asdouble(0x0000000000000000), asdouble(0x0000000000000000)),
    (asdouble(0x3FEFC00000000000), asdouble(0x3F80101575890000), asdouble(0xBD10C76B999D2BE8)),
    (asdouble(0x3FEF800000000000), asdouble(0x3F90205658938000), asdouble(0xBD23DC5B06E2F7D2)),
    (asdouble(0x3FEF400000000000), asdouble(0x3F98492528C90000), asdouble(0xBD2AA0BA325A0C34)),
    (asdouble(0x3FEF000000000000), asdouble(0x3FA0415D89E74000), asdouble(0x3D0111C05CF1D753)),
    (asdouble(0x3FEEC00000000000), asdouble(0x3FA466AED42E0000), asdouble(0xBD2C167375BDFD28)),
    (asdouble(0x3FEE800000000000), asdouble(0x3FA894AA149FC000), asdouble(0xBD197995D05A267D)),
    (asdouble(0x3FEE400000000000), asdouble(0x3FACCB73CDDDC000), asdouble(0xBD1A68F247D82807)),
    (asdouble(0x3FEE200000000000), asdouble(0x3FAEEA31C006C000), asdouble(0xBD0E113E4FC93B7B)),
    (asdouble(0x3FEDE00000000000), asdouble(0x3FB1973BD1466000), asdouble(0xBD25325D560D9E9B)),
    (asdouble(0x3FEDA00000000000), asdouble(0x3FB3BDF5A7D1E000), asdouble(0x3D2CC85EA5DB4ED7)),
    (asdouble(0x3FED600000000000), asdouble(0x3FB5E95A4D97A000), asdouble(0xBD2C69063C5D1D1E)),
    (asdouble(0x3FED400000000000), asdouble(0x3FB700D30AEAC000), asdouble(0x3CEC1E8DA99DED32)),
    (asdouble(0x3FED000000000000), asdouble(0x3FB9335E5D594000), asdouble(0x3D23115C3ABD47DA)),
    (asdouble(0x3FECC00000000000), asdouble(0x3FBB6AC88DAD6000), asdouble(0xBD1390802BF768E5)),
    (asdouble(0x3FECA00000000000), asdouble(0x3FBC885801BC4000), asdouble(0x3D2646D1C65AACD3)),
    (asdouble(0x3FEC600000000000), asdouble(0x3FBEC739830A2000), asdouble(0xBD2DC068AFE645E0)),
    (asdouble(0x3FEC400000000000), asdouble(0x3FBFE89139DBE000), asdouble(0xBD2534D64FA10AFD)),
    (asdouble(0x3FEC000000000000), asdouble(0x3FC1178E8227E000), asdouble(0x3D21EF78CE2D07F2)),
    (asdouble(0x3FEBE00000000000), asdouble(0x3FC1AA2B7E23F000), asdouble(0x3D2CA78E44389934)),
    (asdouble(0x3FEBA00000000000), asdouble(0x3FC2D1610C868000), asdouble(0x3D039D6CCB81B4A1)),
    (asdouble(0x3FEB800000000000), asdouble(0x3FC365FCB0159000), asdouble(0x3CC62FA8234B7289)),
    (asdouble(0x3FEB400000000000), asdouble(0x3FC4913D8333B000), asdouble(0x3D25837954FDB678)),
    (asdouble(0x3FEB200000000000), asdouble(0x3FC527E5E4A1B000), asdouble(0x3D2633E8E5697DC7)),
    (asdouble(0x3FEAE00000000000), asdouble(0x3FC6574EBE8C1000), asdouble(0x3D19CF8B2C3C2E78)),
    (asdouble(0x3FEAC00000000000), asdouble(0x3FC6F0128B757000), asdouble(0xBD25118DE59C21E1)),
    (asdouble(0x3FEAA00000000000), asdouble(0x3FC7898D85445000), asdouble(0xBD1C661070914305)),
    (asdouble(0x3FEA600000000000), asdouble(0x3FC8BEAFEB390000), asdouble(0xBD073D54AAE92CD1)),
    (asdouble(0x3FEA400000000000), asdouble(0x3FC95A5ADCF70000), asdouble(0x3D07F22858A0FF6F)),
    (asdouble(0x3FEA000000000000), asdouble(0x3FCA93ED3C8AE000), asdouble(0xBD28724350562169)),
    (asdouble(0x3FE9E00000000000), asdouble(0x3FCB31D8575BD000), asdouble(0xBD0C358D4EACE1AA)),
    (asdouble(0x3FE9C00000000000), asdouble(0x3FCBD087383BE000), asdouble(0xBD2D4BC4595412B6)),
    (asdouble(0x3FE9A00000000000), asdouble(0x3FCC6FFBC6F01000), asdouble(0xBCF1EC72C5962BD2)),
    (asdouble(0x3FE9600000000000), asdouble(0x3FCDB13DB0D49000), asdouble(0xBD2AFF2AF715B035)),
    (asdouble(0x3FE9400000000000), asdouble(0x3FCE530EFFE71000), asdouble(0x3CC212276041F430)),
    (asdouble(0x3FE9200000000000), asdouble(0x3FCEF5ADE4DD0000), asdouble(0xBCCA211565BB8E11)),
    (asdouble(0x3FE9000000000000), asdouble(0x3FCF991C6CB3B000), asdouble(0x3D1BCBECCA0CDF30)),
    (asdouble(0x3FE8C00000000000), asdouble(0x3FD07138604D5800), asdouble(0x3CF89CDB16ED4E91)),
    (asdouble(0x3FE8A00000000000), asdouble(0x3FD0C42D67616000), asdouble(0x3D27188B163CEAE9)),
    (asdouble(0x3FE8800000000000), asdouble(0x3FD1178E8227E800), asdouble(0xBD2C210E63A5F01C)),
    (asdouble(0x3FE8600000000000), asdouble(0x3FD16B5CCBACF800), asdouble(0x3D2B9ACDF7A51681)),
    (asdouble(0x3FE8400000000000), asdouble(0x3FD1BF99635A6800), asdouble(0x3D2CA6ED5147BDB7)),
    (asdouble(0x3FE8200000000000), asdouble(0x3FD214456D0EB800), asdouble(0x3D0A87DEBA46BAEA)),
    (asdouble(0x3FE7E00000000000), asdouble(0x3FD2BEF07CDC9000), asdouble(0x3D2A9CFA4A5004F4)),
    (asdouble(0x3FE7C00000000000), asdouble(0x3FD314F1E1D36000), asdouble(0xBD28E27AD3213CB8)),
    (asdouble(0x3FE7A00000000000), asdouble(0x3FD36B6776BE1000), asdouble(0x3D116ECDB0F177C8)),
    (asdouble(0x3FE7800000000000), asdouble(0x3FD3C25277333000), asdouble(0x3D183B54B606BD5C)),
    (asdouble(0x3FE7600000000000), asdouble(0x3FD419B423D5E800), asdouble(0x3D08E436EC90E09D)),
    (asdouble(0x3FE7400000000000), asdouble(0x3FD4718DC271C800), asdouble(0xBD2F27CE0967D675)),
    (asdouble(0x3FE7200000000000), asdouble(0x3FD4C9E09E173000), asdouble(0xBD2E20891B0AD8A4)),
    (asdouble(0x3FE7000000000000), asdouble(0x3FD522AE0738A000), asdouble(0x3D2EBE708164C759)),
    (asdouble(0x3FE6E00000000000), asdouble(0x3FD57BF753C8D000), asdouble(0x3D1FADEDEE5D40EF)),
    (asdouble(0x3FE6C00000000000), asdouble(0x3FD5D5BDDF596000), asdouble(0xBD0A0B2A08A465DC)),
];

// ============================================================
// Duplicate exp data for pow's exp_inline (self-contained)
// ============================================================

const POW_EXP_TABLE_BITS: u32 = 7;
const POW_EXP_N: u64 = 1 << POW_EXP_TABLE_BITS;
const POW_SIGN_BIAS: u64 = 0x800 << POW_EXP_TABLE_BITS; // 0x40000

const POW_INVLN2N: f64 = asdouble(0x40671547652B82FE);
const POW_NEGLN2HI_N: f64 = asdouble(0xBF762E42FEFA0000);
const POW_NEGLN2LO_N: f64 = asdouble(0xBD0CF79ABC9E3B3A);
const POW_SHIFT: f64 = asdouble(0x4338000000000000);
const POW_1P1009: f64 = asdouble(0x7F00000000000000);
const POW_1P_NEG1022: f64 = asdouble(0x0010000000000000);

const POW_EXP_C2: f64 = asdouble(0x3FDFFFFFFFFFFDBD);
const POW_EXP_C3: f64 = asdouble(0x3FC555555555543C);
const POW_EXP_C4: f64 = asdouble(0x3FA55555CF172B91);
const POW_EXP_C5: f64 = asdouble(0x3F81111167A4D017);

const POW_EXP_TAB: [u64; 256] = [
    0x0, 0x3ff0000000000000,
    0x3c9b3b4f1a88bf6e, 0x3feff63da9fb3335,
    0xbc7160139cd8dc5d, 0x3fefec9a3e778061,
    0xbc905e7a108766d1, 0x3fefe315e86e7f85,
    0x3c8cd2523567f613, 0x3fefd9b0d3158574,
    0xbc8bce8023f98efa, 0x3fefd06b29ddf6de,
    0x3c60f74e61e6c861, 0x3fefc74518759bc8,
    0x3c90a3e45b33d399, 0x3fefbe3ecac6f383,
    0x3c979aa65d837b6d, 0x3fefb5586cf9890f,
    0x3c8eb51a92fdeffc, 0x3fefac922b7247f7,
    0x3c3ebe3d702f9cd1, 0x3fefa3ec32d3d1a2,
    0xbc6a033489906e0b, 0x3fef9b66affed31b,
    0xbc9556522a2fbd0e, 0x3fef9301d0125b51,
    0xbc5080ef8c4eea55, 0x3fef8abdc06c31cc,
    0xbc91c923b9d5f416, 0x3fef829aaea92de0,
    0x3c80d3e3e95c55af, 0x3fef7a98c8a58e51,
    0xbc801b15eaa59348, 0x3fef72b83c7d517b,
    0xbc8f1ff055de323d, 0x3fef6af9388c8dea,
    0x3c8b898c3f1353bf, 0x3fef635beb6fcb75,
    0xbc96d99c7611eb26, 0x3fef5be084045cd4,
    0x3c9aecf73e3a2f60, 0x3fef54873168b9aa,
    0xbc8fe782cb86389d, 0x3fef4d5022fcd91d,
    0x3c8a6f4144a6c38d, 0x3fef463b88628cd6,
    0x3c807a05b0e4047d, 0x3fef3f49917ddc96,
    0x3c968efde3a8a894, 0x3fef387a6e756238,
    0x3c875e18f274487d, 0x3fef31ce4fb2a63f,
    0x3c80472b981fe7f2, 0x3fef2b4565e27cdd,
    0xbc96b87b3f71085e, 0x3fef24dfe1f56381,
    0x3c82f7e16d09ab31, 0x3fef1e9df51fdee1,
    0xbc3d219b1a6fbffa, 0x3fef187fd0dad990,
    0x3c8b3782720c0ab4, 0x3fef1285a6e4030b,
    0x3c6e149289cecb8f, 0x3fef0cafa93e2f56,
    0x3c834d754db0abb6, 0x3fef06fe0a31b715,
    0x3c864201e2ac744c, 0x3fef0170fc4cd831,
    0x3c8fdd395dd3f84a, 0x3feefc08b26416ff,
    0xbc86a3803b8e5b04, 0x3feef6c55f929ff1,
    0xbc924aedcc4b5068, 0x3feef1a7373aa9cb,
    0xbc9907f81b512d8e, 0x3feeecae6d05d866,
    0xbc71d1e83e9436d2, 0x3feee7db34e59ff7,
    0xbc991919b3ce1b15, 0x3feee32dc313a8e5,
    0x3c859f48a72a4c6d, 0x3feedea64c123422,
    0xbc9312607a28698a, 0x3feeda4504ac801c,
    0xbc58a78f4817895b, 0x3feed60a21f72e2a,
    0xbc7c2c9b67499a1b, 0x3feed1f5d950a897,
    0x3c4363ed60c2ac11, 0x3feece086061892d,
    0x3c9666093b0664ef, 0x3feeca41ed1d0057,
    0x3c6ecce1daa10379, 0x3feec6a2b5c13cd0,
    0x3c93ff8e3f0f1230, 0x3feec32af0d7d3de,
    0x3c7690cebb7aafb0, 0x3feebfdad5362a27,
    0x3c931dbdeb54e077, 0x3feebcb299fddd0d,
    0xbc8f94340071a38e, 0x3feeb9b2769d2ca7,
    0xbc87deccdc93a349, 0x3feeb6daa2cf6642,
    0xbc78dec6bd0f385f, 0x3feeb42b569d4f82,
    0xbc861246ec7b5cf6, 0x3feeb1a4ca5d920f,
    0x3c93350518fdd78e, 0x3feeaf4736b527da,
    0x3c7b98b72f8a9b05, 0x3feead12d497c7fd,
    0x3c9063e1e21c5409, 0x3feeab07dd485429,
    0x3c34c7855019c6ea, 0x3feea9268a5946b7,
    0x3c9432e62b64c035, 0x3feea76f15ad2148,
    0xbc8ce44a6199769f, 0x3feea5e1b976dc09,
    0xbc8c33c53bef4da8, 0x3feea47eb03a5585,
    0xbc845378892be9ae, 0x3feea34634ccc320,
    0xbc93cedd78565858, 0x3feea23882552225,
    0x3c5710aa807e1964, 0x3feea155d44ca973,
    0xbc93b3efbf5e2228, 0x3feea09e667f3bcd,
    0xbc6a12ad8734b982, 0x3feea012750bdabf,
    0xbc6367efb86da9ee, 0x3fee9fb23c651a2f,
    0xbc80dc3d54e08851, 0x3fee9f7df9519484,
    0xbc781f647e5a3ecf, 0x3fee9f75e8ec5f74,
    0xbc86ee4ac08b7db0, 0x3fee9f9a48a58174,
    0xbc8619321e55e68a, 0x3fee9feb564267c9,
    0x3c909ccb5e09d4d3, 0x3feea0694fde5d3f,
    0xbc7b32dcb94da51d, 0x3feea11473eb0187,
    0x3c94ecfd5467c06b, 0x3feea1ed0130c132,
    0x3c65ebe1abd66c55, 0x3feea2f336cf4e62,
    0xbc88a1c52fb3cf42, 0x3feea427543e1a12,
    0xbc9369b6f13b3734, 0x3feea589994cce13,
    0xbc805e843a19ff1e, 0x3feea71a4623c7ad,
    0xbc94d450d872576e, 0x3feea8d99b4492ed,
    0x3c90ad675b0e8a00, 0x3feeaac7d98a6699,
    0x3c8db72fc1f0eab4, 0x3feeace5422aa0db,
    0xbc65b6609cc5e7ff, 0x3feeaf3216b5448c,
    0x3c7bf68359f35f44, 0x3feeb1ae99157736,
    0xbc93091fa71e3d83, 0x3feeb45b0b91ffc6,
    0xbc5da9b88b6c1e29, 0x3feeb737b0cdc5e5,
    0xbc6c23f97c90b959, 0x3feeba44cbc8520f,
    0xbc92434322f4f9aa, 0x3feebd829fde4e50,
    0xbc85ca6cd7668e4b, 0x3feec0f170ca07ba,
    0x3c71affc2b91ce27, 0x3feec49182a3f090,
    0x3c6dd235e10a73bb, 0x3feec86319e32323,
    0xbc87c50422622263, 0x3feecc667b5de565,
    0x3c8b1c86e3e231d5, 0x3feed09bec4a2d33,
    0xbc91bbd1d3bcbb15, 0x3feed503b23e255d,
    0x3c90cc319cee31d2, 0x3feed99e1330b358,
    0x3c8469846e735ab3, 0x3feede6b5579fdbf,
    0xbc82dfcd978e9db4, 0x3feee36bbfd3f37a,
    0x3c8c1a7792cb3387, 0x3feee89f995ad3ad,
    0xbc907b8f4ad1d9fa, 0x3feeee07298db666,
    0xbc55c3d956dcaeba, 0x3feef3a2b84f15fb,
    0xbc90a40e3da6f640, 0x3feef9728de5593a,
    0xbc68d6f438ad9334, 0x3feeff76f2fb5e47,
    0xbc91eee26b588a35, 0x3fef05b030a1064a,
    0x3c74ffd70a5fddcd, 0x3fef0c1e904bc1d2,
    0xbc91bdfbfa9298ac, 0x3fef12c25bd71e09,
    0x3c736eae30af0cb3, 0x3fef199bdd85529c,
    0x3c8ee3325c9ffd94, 0x3fef20ab5fffd07a,
    0x3c84e08fd10959ac, 0x3fef27f12e57d14b,
    0x3c63cdaf384e1a67, 0x3fef2f6d9406e7b5,
    0x3c676b2c6c921968, 0x3fef3720dcef9069,
    0xbc808a1883ccb5d2, 0x3fef3f0b555dc3fa,
    0xbc8fad5d3ffffa6f, 0x3fef472d4a07897c,
    0xbc900dae3875a949, 0x3fef4f87080d89f2,
    0x3c74a385a63d07a7, 0x3fef5818dcfba487,
    0xbc82919e2040220f, 0x3fef60e316c98398,
    0x3c8e5a50d5c192ac, 0x3fef69e603db3285,
    0x3c843a59ac016b4b, 0x3fef7321f301b460,
    0xbc82d52107b43e1f, 0x3fef7c97337b9b5f,
    0xbc892ab93b470dc9, 0x3fef864614f5a129,
    0x3c74b604603a88d3, 0x3fef902ee78b3ff6,
    0x3c83c5ec519d7271, 0x3fef9a51fbc74c83,
    0xbc8ff7128fd391f0, 0x3fefa4afa2a490da,
    0xbc8dae98e223747d, 0x3fefaf482d8e67f1,
    0x3c8ec3bc41aa2008, 0x3fefba1bee615a27,
    0x3c842b94c3a9eb32, 0x3fefc52b376bba97,
    0x3c8a64a931d185ee, 0x3fefd0765b6e4540,
    0xbc8e37bae43be3ed, 0x3fefdbfdad9cbe14,
    0x3c77893b4d91cd9d, 0x3fefe7c1819e90d8,
    0x3c5305c14160cc89, 0x3feff3c22b8f71f1,
];

// ============================================================
// pow helpers
// ============================================================

/// Top 12 bits of a double (sign and exponent bits).
#[inline]
fn pow_top12(x: f64) -> u32 {
    (asuint64(x) >> 52) as u32
}

/// Compute y+tail = log(x) where the rounded result is y and tail has about
/// additional 15 bits precision. IX is the bit representation of x, but
/// normalized in the subnormal range using the sign bit for the exponent.
/// Uses non-FMA path.
#[inline]
fn pow_log_inline(ix: u64, tail: &mut f64) -> f64 {
    let tmp = ix.wrapping_sub(POW_OFF);
    let i = ((tmp >> (52 - POW_LOG_TABLE_BITS)) % POW_N) as usize;
    let k = (tmp as i64 >> 52) as i32;
    let iz = ix - (tmp & (0xFFFu64 << 52));
    let z = asdouble(iz);
    let kd = k as f64;

    let invc = POW_LOG_TAB[i].0;
    let logc = POW_LOG_TAB[i].1;
    let logctail = POW_LOG_TAB[i].2;

    // Non-FMA path: split z for accurate r = z/c - 1
    let zhi = asdouble((iz.wrapping_add(1u64 << 31)) & (!0u64 << 32));
    let zlo = z - zhi;
    let rhi = zhi * invc - 1.0;
    let rlo = zlo * invc;
    let r = rhi + rlo;

    // k*Ln2 + log(c) + r
    let t1 = kd * POW_LN2HI + logc;
    let t2 = t1 + r;
    let lo1 = kd * POW_LN2LO + logctail;
    let lo2 = t1 - t2 + r;

    // Polynomial: p = log1p(r) - r - A[0]*r^2
    let ar = POW_POLY[0] * r;
    let ar2 = r * ar;
    let ar3 = r * ar2;

    // Non-FMA path for hi/lo3/lo4
    let arhi = POW_POLY[0] * rhi;
    let arhi2 = rhi * arhi;
    let hi = t2 + arhi2;
    let lo3 = rlo * (ar + arhi);
    let lo4 = t2 - hi + arhi2;

    let p = ar3 * (POW_POLY[1]
        + r * POW_POLY[2]
        + ar2 * (POW_POLY[3] + r * POW_POLY[4] + ar2 * (POW_POLY[5] + r * POW_POLY[6])));
    let lo = lo1 + lo2 + lo3 + lo4 + p;
    let y = hi + lo;
    *tail = hi - y + lo;
    y
}

/// Handle cases that may overflow or underflow when computing the result
/// that is scale*(1+tmp) without intermediate rounding.
#[inline]
fn pow_exp_specialcase(tmp: f64, sbits: u64, ki: u64) -> f64 {
    if (ki & 0x80000000) == 0 {
        // k > 0, the exponent of scale might have overflowed by <= 460.
        let sbits = sbits.wrapping_sub(1009u64 << 52);
        let scale = asdouble(sbits);
        let y = POW_1P1009 * (scale + scale * tmp);
        return eval_as_double(y);
    }
    // k < 0, need special care in the subnormal range.
    let sbits = sbits.wrapping_add(1022u64 << 52);
    let scale = asdouble(sbits);
    let mut y = scale + scale * tmp;
    if y.abs() < 1.0 {
        // Round y to the right precision before scaling into subnormal range.
        let mut lo = scale - y + scale * tmp;
        let mut one: f64 = 1.0;
        if y < 0.0 {
            one = -1.0;
        }
        let hi = one + y;
        lo = one - hi + y + lo;
        y = eval_as_double(hi + lo) - one;
        // Fix the sign of 0.
        if y == 0.0 {
            y = asdouble(sbits & 0x8000000000000000);
            // The rounded result is 0, so the final multiplication would be
            // exact and would not signal underflow. Signal it explicitly.
            fp_force_eval(fp_barrier(POW_1P_NEG1022) * POW_1P_NEG1022);
        } else {
            // If the scaled result is inexact-subnormal or underflows to 0,
            // the multiplication raises UNDERFLOW; if it is exact, it does not.
            fp_force_eval(POW_1P_NEG1022 * y);
        }
    }
    y = POW_1P_NEG1022 * y;
    eval_as_double(y)
}

/// Computes sign*exp(x+xtail) where |xtail| < 2^-8/N and |xtail| <= |x|.
/// The sign_bias argument is POW_SIGN_BIAS or 0 and sets the sign to -1 or 1.
#[inline]
fn pow_exp_inline(x: f64, xtail: f64, sign_bias: u64) -> f64 {
    let mut abstop = pow_top12(x) & 0x7ff;

    if predict_false(
        abstop.wrapping_sub(0x3C9) >= 0x408 - 0x3C9,
    ) {
        if abstop.wrapping_sub(0x3C9) >= 0x80000000 {
            // Avoid spurious underflow for tiny x. Note: 0 is common input.
            let one: f64 = 1.0 + x;
            return if sign_bias != 0 { -one } else { one };
        }
        if abstop >= 0x409 {
            // Note: inf and nan are already handled.
            if asuint64(x) >> 63 != 0 {
                return __math_uflow(sign_bias as u32);
            } else {
                return __math_oflow(sign_bias as u32);
            }
        }
        // Large x is special cased below.
        abstop = 0;
    }

    // exp(x) = 2^(k/N) * exp(r), with exp(r) in [2^(-1/2N),2^(1/2N)].
    // x = ln2/N*k + r, with int k and r in [-ln2/2N, ln2/2N].
    let z = POW_INVLN2N * x;
    // z - kd is in [-1, 1] in non-nearest rounding modes.
    let kd = eval_as_double(z + POW_SHIFT);
    let ki = asuint64(kd);
    let kd = kd - POW_SHIFT;
    let mut r = x + kd * POW_NEGLN2HI_N + kd * POW_NEGLN2LO_N;
    // The code assumes 2^-200 < |xtail| < 2^-8/N.
    r += xtail;
    // 2^(k/N) ~= scale * (1 + tail).
    let idx = (2 * (ki % POW_EXP_N)) as usize;
    let top = (ki.wrapping_add(sign_bias)) << (52 - POW_EXP_TABLE_BITS);
    let tail = asdouble(POW_EXP_TAB[idx]);
    // This is only a valid scale when -1023*N < k < 1024*N.
    let sbits = POW_EXP_TAB[idx + 1].wrapping_add(top);
    // exp(x) = 2^(k/N) * exp(r) ~= scale + scale * (tail + exp(r) - 1).
    let r2 = r * r;
    // Worst case error is less than 0.5+1.11/N+(abs poly error * 2^53) ulp.
    let tmp = tail + r + r2 * (POW_EXP_C2 + r * POW_EXP_C3) + r2 * r2 * (POW_EXP_C4 + r * POW_EXP_C5);
    if predict_false(abstop == 0) {
        return pow_exp_specialcase(tmp, sbits, ki);
    }
    let scale = asdouble(sbits);
    // Note: tmp == 0 or |tmp| > 2^-200 and scale > 2^-739, so there
    // is no spurious underflow here even without fma.
    eval_as_double(scale + scale * tmp)
}

/// Returns 0 if not int, 1 if odd int, 2 if even int.
/// The argument is the bit representation of a non-zero finite floating-point value.
#[inline]
fn pow_checkint(iy: u64) -> i32 {
    let e = (iy >> 52) & 0x7ff;
    if e < 0x3ff {
        return 0;
    }
    if e > 0x3ff + 52 {
        return 2;
    }
    if (iy & ((1u64 << (0x3ff + 52 - e)) - 1)) != 0 {
        return 0;
    }
    if (iy & (1u64 << (0x3ff + 52 - e))) != 0 {
        return 1;
    }
    2
}

/// Returns true if input is the bit representation of 0, infinity or nan.
#[inline]
fn pow_zeroinfnan(i: u64) -> bool {
    2u64.wrapping_mul(i).wrapping_sub(1) >= 2u64.wrapping_mul(asuint64(f64::INFINITY)).wrapping_sub(1)
}

// ============================================================
// pow (double precision)
// ============================================================

#[no_mangle]
pub extern "C" fn pow(x: f64, y: f64) -> f64 {
    let mut sign_bias: u64 = 0;
    let ix = asuint64(x);
    let iy = asuint64(y);
    let mut topx = pow_top12(x);
    let topy = pow_top12(y);

    // Exact identity: x^1 is x for all finite/infinite/NaN x, with no exceptions.
    if iy == asuint64(1.0) {
        return x;
    }
    // Exact identity: x^-1 is 1/x, letting the division raise the correct flags.
    if iy == asuint64(-1.0) {
        return 1.0 / x;
    }

    if predict_false(
        topx.wrapping_sub(0x001) >= 0x7ff - 0x001
            || (topy & 0x7ff).wrapping_sub(0x3be) >= 0x43e - 0x3be,
    ) {
        // Note: if |y| > 1075*ln2*2^53 then pow(x,y) = inf/0
        // and if |y| < 2^-54/1075 then pow(x,y) = +-1.
        // Special cases: (x < 0x1p-126 or inf or nan) or
        // (|y| < 0x1p-65 or |y| >= 0x1p63 or nan).
        if predict_false(pow_zeroinfnan(iy)) {
            if 2u64.wrapping_mul(iy) == 0 {
                // issignaling_inline(x) is always false
                return 1.0;
            }
            if ix == asuint64(1.0) {
                // issignaling_inline(y) is always false
                return 1.0;
            }
            if 2u64.wrapping_mul(ix) > 2u64.wrapping_mul(asuint64(f64::INFINITY))
                || 2u64.wrapping_mul(iy) > 2u64.wrapping_mul(asuint64(f64::INFINITY))
            {
                return x + y;
            }
            if 2u64.wrapping_mul(ix) == 2u64.wrapping_mul(asuint64(1.0)) {
                return 1.0;
            }
            if (2u64.wrapping_mul(ix) < 2u64.wrapping_mul(asuint64(1.0)))
                == !(iy >> 63 != 0)
            {
                return 0.0; // |x|<1 && y==inf or |x|>1 && y==-inf.
            }
            return y * y;
        }
        if predict_false(pow_zeroinfnan(ix)) {
            let mut x2 = x * x;
            if ix >> 63 != 0 && pow_checkint(iy) == 1 {
                x2 = -x2;
            }
            // Without the barrier some versions of clang hoist the 1/x2 and
            // thus division by zero exception can be signaled spuriously.
            return if iy >> 63 != 0 {
                fp_barrier(1.0 / x2)
            } else {
                x2
            };
        }
        // Here x and y are non-zero finite.
        let mut ix = ix;
        if ix >> 63 != 0 {
            // Finite x < 0.
            let yint = pow_checkint(iy);
            if yint == 0 {
                return __math_invalid(x);
            }
            if yint == 1 {
                sign_bias = POW_SIGN_BIAS;
            }
            ix &= 0x7fffffffffffffff;
            topx &= 0x7ff;
        }
        if (topy & 0x7ff).wrapping_sub(0x3be) >= 0x43e - 0x3be {
            // Note: sign_bias == 0 here because y is not odd.
            if ix == asuint64(1.0) {
                return 1.0;
            }
            if (topy & 0x7ff) < 0x3be {
                // |y| < 2^-65, x^y ~= 1 + y*log(x).
                return if ix > asuint64(1.0) {
                    1.0 + y
                } else {
                    1.0 - y
                };
            }
            return if (ix > asuint64(1.0)) == (topy < 0x800) {
                __math_oflow(0)
            } else {
                __math_uflow(0)
            };
        }
        if topx == 0 {
            // Normalize subnormal x so exponent becomes negative.
            let mut ix = asuint64(x * asdouble(0x4330000000000000)); // 0x1p52
            ix &= 0x7fffffffffffffff;
            ix = ix.wrapping_sub(52u64 << 52);
            // Fall through to log_inline/exp_inline with normalized ix.
            let mut lo: f64 = 0.0;
            let hi = pow_log_inline(ix, &mut lo);
            let yhi = asdouble(iy & (!0u64 << 27));
            let ylo = y - yhi;
            let lhi = asdouble(asuint64(hi) & (!0u64 << 27));
            let llo = hi - lhi + lo;
            let ehi = yhi * lhi;
            let elo = ylo * lhi + y * llo;
            return pow_exp_inline(ehi, elo, sign_bias);
        }
        // Non-subnormal, non-special x.
        let mut lo: f64 = 0.0;
        let hi = pow_log_inline(ix, &mut lo);
        let yhi = asdouble(iy & (!0u64 << 27));
        let ylo = y - yhi;
        let lhi = asdouble(asuint64(hi) & (!0u64 << 27));
        let llo = hi - lhi + lo;
        let ehi = yhi * lhi;
        let elo = ylo * lhi + y * llo;
        return pow_exp_inline(ehi, elo, sign_bias);
    }

    // Fast path: both x and y are normal, non-special.
    let mut lo: f64 = 0.0;
    let hi = pow_log_inline(ix, &mut lo);
    let yhi = asdouble(iy & (!0u64 << 27));
    let ylo = y - yhi;
    let lhi = asdouble(asuint64(hi) & (!0u64 << 27));
    let llo = hi - lhi + lo;
    let ehi = yhi * lhi;
    let elo = ylo * lhi + y * llo;
    pow_exp_inline(ehi, elo, sign_bias)
}

// ============================================================
// powf (single precision) data
// ============================================================

const POWF_LOG2_TABLE_BITS: u32 = 4;
const POWF_N: u64 = 1 << POWF_LOG2_TABLE_BITS;
const POWF_OFF: u32 = 0x3F330000;

const POWF_LOG2_TAB: [(f64, f64); 16] = [
    (asdouble(0x3FF661EC79F8F3BE), asdouble(0xBFDEFEC65B963019)),
    (asdouble(0x3FF571ED4AAF883D), asdouble(0xBFDB0B6832D4FCA4)),
    (asdouble(0x3FF49539F0F010B0), asdouble(0xBFD7418B0A1FB77B)),
    (asdouble(0x3FF3C995B0B80385), asdouble(0xBFD39DE91A6DCF7B)),
    (asdouble(0x3FF30D190C8864A5), asdouble(0xBFD01D9BF3F2B631)),
    (asdouble(0x3FF25E227B0B8EA0), asdouble(0xBFC97C1D1B3B7AF0)),
    (asdouble(0x3FF1BB4A4A1A343F), asdouble(0xBFC2F9E393AF3C9F)),
    (asdouble(0x3FF12358F08AE5BA), asdouble(0xBFB960CBBF788D5C)),
    (asdouble(0x3FF0953F419900A7), asdouble(0xBFAA6F9DB6475FCE)),
    (asdouble(0x3FF0000000000000), asdouble(0x0000000000000000)),
    (asdouble(0x3FEE608CFD9A47AC), asdouble(0x3FB338CA9F24F53D)),
    (asdouble(0x3FECA4B31F026AA0), asdouble(0x3FC476A9543891BA)),
    (asdouble(0x3FEB2036576AFCE6), asdouble(0x3FCE840B4AC4E4D2)),
    (asdouble(0x3FE9C2D163A1AA2D), asdouble(0x3FD40645F0C6651C)),
    (asdouble(0x3FE886E6037841ED), asdouble(0x3FD88E9C2C1B9FF8)),
    (asdouble(0x3FE767DCF5534862), asdouble(0x3FDCE0A44EB17BCC)),
];

const POWF_LOG2_POLY: [f64; 5] = [
    asdouble(0x3FD27616C9496E0B),
    asdouble(0xBFD71969A075C67A),
    asdouble(0x3FDEC70A6CA7BADD),
    asdouble(0xBFE7154748BEF6C8),
    asdouble(0x3FF71547652AB82B),
];

// ============================================================
// Duplicate exp2f data for powf's exp2_inline (self-contained)
// ============================================================

const POWF_EXP2F_TABLE_BITS: u32 = 5;
const POWF_N_EXP2F: u64 = 1 << POWF_EXP2F_TABLE_BITS;
const POWF_SIGN_BIAS: u64 = 1 << (POWF_EXP2F_TABLE_BITS + 11); // 0x10000

const POWF_EXP2F_SHIFT_SCALED: f64 = asdouble(0x42E8000000000000);
const POWF_EXP2F_POLY: [f64; 3] = [
    asdouble(0x3FAC6AF84B912394),
    asdouble(0x3FCEBFCE50FAC4F3),
    asdouble(0x3FE62E42FF0C52D6),
];

const POWF_EXP2F_TAB: [u64; 32] = [
    0x3ff0000000000000, 0x3fefd9b0d3158574, 0x3fefb5586cf9890f, 0x3fef9301d0125b51,
    0x3fef72b83c7d517b, 0x3fef54873168b9aa, 0x3fef387a6e756238, 0x3fef1e9df51fdee1,
    0x3fef06fe0a31b715, 0x3feef1a7373aa9cb, 0x3feedea64c123422, 0x3feece086061892d,
    0x3feebfdad5362a27, 0x3feeb42b569d4f82, 0x3feeab07dd485429, 0x3feea47eb03a5585,
    0x3feea09e667f3bcd, 0x3fee9f75e8ec5f74, 0x3feea11473eb0187, 0x3feea589994cce13,
    0x3feeace5422aa0db, 0x3feeb737b0cdc5e5, 0x3feec49182a3f090, 0x3feed503b23e255d,
    0x3feee89f995ad3ad, 0x3feeff76f2fb5e47, 0x3fef199bdd85529c, 0x3fef3720dcef9069,
    0x3fef5818dcfba487, 0x3fef7c97337b9b5f, 0x3fefa4afa2a490da, 0x3fefd0765b6e4540,
];

// ============================================================
// powf helpers
// ============================================================

/// log2(x) for powf. IX is the bit representation of x, normalized for subnormals.
#[inline]
fn powf_log2_inline(ix: u32) -> f64 {
    let tmp = ix.wrapping_sub(POWF_OFF);
    let i = ((tmp >> (23 - POWF_LOG2_TABLE_BITS)) % (POWF_N as u32)) as usize;
    let _top = tmp & 0xff800000;
    let iz = ix - _top;
    let k = (tmp as i32 >> 23) as i32;
    let invc = POWF_LOG2_TAB[i].0;
    let logc = POWF_LOG2_TAB[i].1;
    let z = asfloat(iz) as f64;

    // log2(x) = log1p(z/c-1)/ln2 + log2(c) + k
    let r = z * invc - 1.0;
    let y0 = logc + k as f64;

    // Pipelined polynomial evaluation to approximate log1p(r)/ln2.
    let r2 = r * r;
    let y = POWF_LOG2_POLY[0] * r + POWF_LOG2_POLY[1];
    let p = POWF_LOG2_POLY[2] * r + POWF_LOG2_POLY[3];
    let r4 = r2 * r2;
    let q = POWF_LOG2_POLY[4] * r + y0;
    let q = p * r2 + q;
    let y = y * r4 + q;
    y
}

/// exp2(xd) for powf. xd is double precision; sign_bias sets the sign of the result.
#[inline]
fn powf_exp2_inline(xd: f64, sign_bias: u64) -> f32 {
    // x = k/N + r with r in [-1/(2N), 1/(2N)]
    let kd = eval_as_double(xd + POWF_EXP2F_SHIFT_SCALED);
    let ki = asuint64(kd);
    let kd = kd - POWF_EXP2F_SHIFT_SCALED; // k/N
    let r = xd - kd;

    // exp2(x) = 2^(k/N) * 2^r ~= s * (C0*r^3 + C1*r^2 + C2*r + 1)
    let t = POWF_EXP2F_TAB[(ki % POWF_N_EXP2F) as usize]
        .wrapping_add((ki.wrapping_add(sign_bias)) << (52 - POWF_EXP2F_TABLE_BITS));
    let s = asdouble(t);
    let z = POWF_EXP2F_POLY[0] * r + POWF_EXP2F_POLY[1];
    let r2 = r * r;
    let y = POWF_EXP2F_POLY[2] * r + 1.0;
    let y = z * r2 + y;
    let y = y * s;
    eval_as_float(y as f32)
}

/// Returns 0 if not int, 1 if odd int, 2 if even int.
#[inline]
fn powf_checkint(iy: u32) -> i32 {
    let e = (iy >> 23) & 0xff;
    if e < 0x7f {
        return 0;
    }
    if e > 0x7f + 23 {
        return 2;
    }
    if (iy & ((1u32 << (0x7f + 23 - e)) - 1)) != 0 {
        return 0;
    }
    if (iy & (1u32 << (0x7f + 23 - e))) != 0 {
        return 1;
    }
    2
}

/// Returns true if input is the bit representation of 0, infinity or nan.
#[inline]
fn powf_zeroinfnan(ix: u32) -> bool {
    2u32.wrapping_mul(ix).wrapping_sub(1) >= 2u32.wrapping_mul(0x7f800000u32).wrapping_sub(1)
}

// ============================================================
// powf (single precision)
// ============================================================

const POWF_OVFL_THRESHOLD: f64 = asdouble(0x405FFFFFFFD1D571);
const POWF_UVFL_THRESHOLD: f64 = asdouble(0xC062C00000000000);

#[no_mangle]
pub extern "C" fn powf(x: f32, y: f32) -> f32 {
    let mut sign_bias: u64 = 0;
    let mut ix = asuint(x);
    let iy = asuint(y);

    // Exact identity: x^1 is x for all finite/infinite/NaN x, with no exceptions.
    if iy == 0x3f800000 {
        return x;
    }
    // Exact identity: x^-1 is 1/x, letting the division raise the correct flags.
    if iy == 0xbf800000 {
        return 1.0 / x;
    }

    if predict_false(
        ix.wrapping_sub(0x00800000) >= 0x7f800000 - 0x00800000 || powf_zeroinfnan(iy),
    ) {
        // Either (x < 0x1p-126 or inf or nan) or (y is 0 or inf or nan).
        if predict_false(powf_zeroinfnan(iy)) {
            if 2u32.wrapping_mul(iy) == 0 {
                // issignalingf_inline(x) is always false
                return 1.0;
            }
            if ix == 0x3f800000 {
                // issignalingf_inline(y) is always false
                return 1.0;
            }
            if 2u32.wrapping_mul(ix) > 2u32.wrapping_mul(0x7f800000u32)
                || 2u32.wrapping_mul(iy) > 2u32.wrapping_mul(0x7f800000u32)
            {
                return x + y;
            }
            if 2u32.wrapping_mul(ix) == 2u32.wrapping_mul(0x3f800000u32) {
                return 1.0;
            }
            if (2u32.wrapping_mul(ix) < 2u32.wrapping_mul(0x3f800000u32))
                == !(iy & 0x80000000 != 0)
            {
                return 0.0; // |x|<1 && y==inf or |x|>1 && y==-inf.
            }
            return y * y;
        }
        if predict_false(powf_zeroinfnan(ix)) {
            let mut x2 = x * x;
            if ix & 0x80000000 != 0 && powf_checkint(iy) == 1 {
                x2 = -x2;
            }
            return if iy & 0x80000000 != 0 {
                fp_barrierf(1.0 / x2)
            } else {
                x2
            };
        }
        // x and y are non-zero finite.
        if ix & 0x80000000 != 0 {
            // Finite x < 0.
            let yint = powf_checkint(iy);
            if yint == 0 {
                return __math_invalidf(x);
            }
            if yint == 1 {
                sign_bias = POWF_SIGN_BIAS;
            }
            ix &= 0x7fffffff;
        }
        if ix < 0x00800000 {
            // Normalize subnormal x so exponent becomes negative.
            ix = asuint(x * asfloat(0x4B000000)); // 0x1p23f
            ix &= 0x7fffffff;
            ix = ix.wrapping_sub(23u32 << 23);
        }
    }

    let logx = powf_log2_inline(ix);
    let ylogx = y as f64 * logx; // cannot overflow, y is single prec.
    if predict_false(
        (asuint64(ylogx) >> 47 & 0xffff) >= (asuint64(126.0) >> 47),
    ) {
        // |y*log(x)| >= 126.
        if ylogx > POWF_OVFL_THRESHOLD {
            return __math_oflowf(sign_bias as u32);
        }
        if ylogx <= POWF_UVFL_THRESHOLD {
            return __math_uflowf(sign_bias as u32);
        }
    }
    powf_exp2_inline(ylogx, sign_bias)
}
