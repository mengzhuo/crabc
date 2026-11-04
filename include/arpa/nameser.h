#ifndef _ARPA_NAMESER_H
#define _ARPA_NAMESER_H

#ifdef __cplusplus
extern "C" {
#endif

int dn_expand(const unsigned char *, const unsigned char *, const unsigned char *, char *, int);

#ifdef __cplusplus
}
#endif

#endif

#include <stddef.h>
#include <stdint.h>

#define NS_PACKETSZ     512
#define NS_MAXDNAME     1025
#define NS_MAXMSG       65535
#define NS_MAXCDNAME    255
#define NS_MAXLABEL     63
#define NS_HFIXEDSZ     12
#define NS_QFIXEDSZ     4
#define NS_RRFIXEDSZ    10
#define NS_INT32SZ      4
#define NS_INT16SZ      2
#define NS_INT8SZ       1
#define NS_INADDRSZ     4
#define NS_IN6ADDRSZ    16
#define NS_CMPRSFLGS    0xc0
#define NS_DEFAULTPORT  53

typedef enum __ns_type {
	ns_t_invalid = 0,
	ns_t_a = 1,
	ns_t_ns = 2,
	ns_t_md = 3,
	ns_t_mf = 4,
	ns_t_cname = 5,
	ns_t_soa = 6,
	ns_t_mb = 7,
	ns_t_mg = 8,
	ns_t_mr = 9,
	ns_t_null = 10,
	ns_t_wks = 11,
	ns_t_ptr = 12,
	ns_t_hinfo = 13,
	ns_t_minfo = 14,
	ns_t_mx = 15,
	ns_t_txt = 16,
	ns_t_rp = 17,
	ns_t_afsdb = 18,
	ns_t_x25 = 19,
	ns_t_isdn = 20,
	ns_t_rt = 21,
	ns_t_nsap = 22,
	ns_t_nsap_ptr = 23,
	ns_t_sig = 24,
	ns_t_key = 25,
	ns_t_px = 26,
	ns_t_gpos = 27,
	ns_t_aaaa = 28,
	ns_t_loc = 29,
	ns_t_nxt = 30,
	ns_t_eid = 31,
	ns_t_nimloc = 32,
	ns_t_srv = 33,
	ns_t_atma = 34,
	ns_t_naptr = 35,
	ns_t_kx = 36,
	ns_t_cert = 37,
	ns_t_a6 = 38,
	ns_t_dname = 39,
	ns_t_sink = 40,
	ns_t_opt = 41,
	ns_t_apl = 42,
	ns_t_ds = 43,
	ns_t_sshfp = 44,
	ns_t_ipseckey = 45,
	ns_t_rrsig = 46,
	ns_t_nsec = 47,
	ns_t_dnskey = 48,
	ns_t_dhcid = 49,
	ns_t_nsec3 = 50,
	ns_t_nsec3param = 51,
	ns_t_tlsa = 52,
	ns_t_smimea = 53,
	ns_t_hip = 55,
	ns_t_ninfo = 56,
	ns_t_rkey = 57,
	ns_t_talink = 58,
	ns_t_cds = 59,
	ns_t_cdnskey = 60,
	ns_t_openpgpkey = 61,
	ns_t_csync = 62,
	ns_t_spf = 99,
	ns_t_uinfo = 100,
	ns_t_uid = 101,
	ns_t_gid = 102,
	ns_t_unspec = 103,
	ns_t_nid = 104,
	ns_t_l32 = 105,
	ns_t_l64 = 106,
	ns_t_lp = 107,
	ns_t_eui48 = 108,
	ns_t_eui64 = 109,
	ns_t_tkey = 249,
	ns_t_tsig = 250,
	ns_t_ixfr = 251,
	ns_t_axfr = 252,
	ns_t_mailb = 253,
	ns_t_maila = 254,
	ns_t_any = 255,
	ns_t_zxfr = 256,
	ns_t_uri = 256,
	ns_t_caa = 257,
	ns_t_avc = 258,
	ns_t_ta = 32768,
	ns_t_dlv = 32769,
	ns_t_max = 65536
} ns_type;

typedef enum __ns_sect {
    ns_s_qd = 0,
    ns_s_zn = 0,
    ns_s_an = 1,
    ns_s_pr = 1,
    ns_s_ns = 2,
    ns_s_ud = 2,
    ns_s_ar = 3,
    ns_s_max = 4
} ns_sect;

typedef struct __ns_msg {
    const unsigned char *_msg, *_eom;
    uint16_t _id, _flags, _counts[ns_s_max];
    const unsigned char *_sections[ns_s_max];
    ns_sect _sect;
    int _rrnum;
    const unsigned char *_msg_ptr;
} ns_msg;

struct _ns_flagdata { int mask, shift; };
extern const struct _ns_flagdata _ns_flagdata[];

#define ns_msg_id(handle) ((handle)._id + 0)
#define ns_msg_base(handle) ((handle)._msg + 0)
#define ns_msg_end(handle) ((handle)._eom + 0)
#define ns_msg_size(handle) ((handle)._eom - (handle)._msg)
#define ns_msg_count(handle, section) ((handle)._counts[section] + 0)
#define ns_msg_getflag(handle, flag) \
    (((handle)._flags & _ns_flagdata[flag].mask) >> _ns_flagdata[flag].shift)

typedef struct __ns_rr {
    char name[NS_MAXDNAME];
    uint16_t type;
    uint16_t rr_class;
    uint32_t ttl;
    uint16_t rdlength;
    const unsigned char *rdata;
} ns_rr;

#define ns_rr_name(rr)  (((rr).name[0] != '\0') ? (rr).name : ".")
#define ns_rr_type(rr)  ((ns_type)((rr).type + 0))
#define ns_rr_class(rr) ((ns_class)((rr).rr_class + 0))
#define ns_rr_ttl(rr)   ((rr).ttl + 0)
#define ns_rr_rdlen(rr) ((rr).rdlength + 0)
#define ns_rr_rdata(rr) ((rr).rdata + 0)

typedef enum __ns_flag {
    ns_f_qr, ns_f_opcode, ns_f_aa, ns_f_tc, ns_f_rd, ns_f_ra,
    ns_f_z, ns_f_ad, ns_f_cd, ns_f_rcode, ns_f_max
} ns_flag;

unsigned ns_get16(const unsigned char *);
unsigned long ns_get32(const unsigned char *);
void ns_put16(unsigned, unsigned char *);
void ns_put32(unsigned long, unsigned char *);
int ns_initparse(const unsigned char *, int, ns_msg *);
int ns_skiprr(const unsigned char *, const unsigned char *, ns_sect, int);
int ns_parserr(ns_msg *, ns_sect, int, ns_rr *);
int ns_name_uncompress(const unsigned char *, const unsigned char *, const unsigned char *, char *, size_t);
