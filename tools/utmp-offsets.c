// utmp-offsets.c
//
// print field offsets of lastlog, lastlogx, utmp, utmpx.
// compile with:
//     cc utmp-offsets.c -o ./utmp-offsets
//
// in bash, run:
//     (set -eux; rm -fv ./utmp-offsets ./utmp-offsets.out; cc -o ./utmp-offsets utmp-offsets.c; uname -a; ./utmp-offsets | tee ./utmp-offsets.out)
//
// to find relevent header files:
//     find /usr/ /lib/ -type f -name 'utmp*h' -o -name 'btmp*h' -o -name 'wtmp*h' -o -name 'last*h'

// The locations of structs is not reliable.
// For example, on some platforms, there is no header `lastlog.h` but the struct
// `lastlog` is defined in `utmp.h`.
// The following includes and `*_COMPILE` can turn these on and off.
// If there is an include error then comment out the include.
// If there is a compilation error then set the associated `*_COMPILE` to 0
// Lastly, the struct may be defined but some fields may not be defined,
// so comment out the print statement for that field.

#include <sys/time.h>  // some systems require the user to include `time.h` first

// USER MUST COMMENT OUT FAILED INCLUDES

#include <acct.h>
#include <sys/acct.h>
#include <faillog.h>
#include <sys/faillog.h>
#include <lastlog.h>
#include <sys/lastlog.h>
#include <lastlogx.h>
#include <sys/lastlogx.h>
#include <utmp.h>
#include <sys/utmp.h>
#include <utmpx.h>
#include <sys/utmpx.h>
#include <utxdb.h>
#include <sys/utxdb.h>

// USER MUST SET COMPILATION FLAGS TO 0

#define ACCT_COMPILE 1
#define ACCT_V3_COMPILE 1
#define FAILLOG_COMPILE 1
#define LASTLOG_COMPILE 1
#define LASTLOGX_COMPILE 1
#define TIMEVAL_COMPILE 1
#define __TIMEVAL_COMPILE 1
#define UTMP_COMPILE 1
#define UTMPX_COMPILE 1
#define UTXDB_COMPILE 1
#define FUTX_COMPILE 1

#include <paths.h>  // defines _PATH_LASTLOG, _PATH_UTMP, etc.
#include <stddef.h>
#include <stdio.h>

// get the CPU architecture as a string.
// ripped from https://stackoverflow.com/a/66249936/471376
const char *
architecture(void)
{
    #if defined(__x86_64__) || defined(_M_X64)
    return "x86_64";
    #elif defined(i386) || defined(__i386__) || defined(__i386) || defined(_M_IX86)
    return "x86_32";
    #elif defined(__ARM_ARCH_2__)
    return "ARM2";
    #elif defined(__ARM_ARCH_3__) || defined(__ARM_ARCH_3M__)
    return "ARM3";
    #elif defined(__ARM_ARCH_4T__) || defined(__TARGET_ARM_4T)
    return "ARM4T";
    #elif defined(__ARM_ARCH_5_) || defined(__ARM_ARCH_5E_)
    return "ARM5"
    #elif defined(__ARM_ARCH_6T2_) || defined(__ARM_ARCH_6T2_)
    return "ARM6T2";
    #elif defined(__ARM_ARCH_6__) || defined(__ARM_ARCH_6J__) || defined(__ARM_ARCH_6K__) || defined(__ARM_ARCH_6Z__) || defined(__ARM_ARCH_6ZK__)
    return "ARMv6";
    #elif defined(__ARM_ARCH_7__) || defined(__ARM_ARCH_7A__) || defined(__ARM_ARCH_7R__) || defined(__ARM_ARCH_7M__) || defined(__ARM_ARCH_7S__)
    return "ARMv7";
    #elif defined(__ARM_ARCH_7A__) || defined(__ARM_ARCH_7R__) || defined(__ARM_ARCH_7M__) || defined(__ARM_ARCH_7S__)
    return "ARMv7A";
    #elif defined(__ARM_ARCH_7R__) || defined(__ARM_ARCH_7M__) || defined(__ARM_ARCH_7S__)
    return "ARMv7R";
    #elif defined(__ARM_ARCH_7M__)
    return "ARMv7M";
    #elif defined(__ARM_ARCH_7S__)
    return "ARMv7S";
    #elif defined(__aarch64__) || defined(_M_ARM64)
    return "ARM64";
    #elif defined(mips) || defined(__mips__) || defined(__mips)
    return "MIPS";
    #elif defined(__sh__)
    return "SUPERH";
    #elif defined(__powerpc) || defined(__powerpc__) || defined(__powerpc64__) || defined(__POWERPC__) || defined(__ppc__) || defined(__PPC__) || defined(_ARCH_PPC)
    return "POWERPC";
    #elif defined(__PPC64__) || defined(__ppc64__) || defined(_ARCH_PPC64)
    return "POWERPC64";
    #elif defined(__sparc__) || defined(__sparc)
    return "SPARC";
    #elif defined(__m68k__)
    return "M68K";
    #else
    return "UNKNOWN";
    #endif
}

#if ACCT_COMPILE
struct acct acct;
#endif

#if ACCT_V3_COMPILE
struct acct_v3 acct_v3;
#endif

#if FAILLOG_COMPILE
struct faillog faillog;
#endif

#if LASTLOG_COMPILE
struct lastlog lastlog;
#endif

#if LASTLOGX_COMPILE
struct lastlogx lastlogx;
#endif

#if TIMEVAL_COMPILE
struct timeval timeval;
#endif

#if UTMP_COMPILE
struct utmp utmp;
#endif

#if UTMPX_COMPILE
struct utmpx utmpx;
#endif

#if UTXDB_COMPILE
struct utxdb utxdb;
#endif

#if FUTX_COMPILE
struct futx futx;
#endif

int
main(void)
{
    printf("CPU Architecture %s\n\n", architecture());

#ifdef ACCT_COMM
    printf("ACCT_COMM %d\n", ACCT_COMM);
#endif

#ifdef AFORK
    printf("AFORK %d\n", AFORK);
#endif
#ifdef ASU
    printf("ASU %d\n", ASU);
#endif
#ifdef ACOMPAT
    printf("ACOMPAT %d\n", ACOMPAT);
#endif
#ifdef ACORE
    printf("ACORE %d\n", ACORE);
#endif
#ifdef AXSIG
    printf("AXSIG %d\n", AXSIG);
#endif

#if ACCT_COMPILE
    printf("\n");
    printf("acct                 sizeof %3lu\n", sizeof(acct));
    printf("acct.ac_flag    @%3lu sizeof %3lu\n", offsetof(struct acct, ac_flag), sizeof(acct.ac_flag));
    printf("acct.ac_version @%3lu sizeof %3lu\n", offsetof(struct acct, ac_version), sizeof(acct.ac_version));
    printf("acct.ac_uid16   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_uid16), sizeof(acct.ac_uid16));
    printf("acct.ac_uid     @%3lu sizeof %3lu\n", offsetof(struct acct, ac_uid), sizeof(acct.ac_uid));
    printf("acct.ac_gid16   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_gid16), sizeof(acct.ac_gid16));
    printf("acct.ac_gid     @%3lu sizeof %3lu\n", offsetof(struct acct, ac_gid), sizeof(acct.ac_gid));
    printf("acct.ac_tty     @%3lu sizeof %3lu\n", offsetof(struct acct, ac_tty), sizeof(acct.ac_tty));
    printf("acct.ac_btime   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_btime), sizeof(acct.ac_btime));
    printf("acct.ac_utime   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_utime), sizeof(acct.ac_utime));
    printf("acct.ac_stime   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_stime), sizeof(acct.ac_stime));
    printf("acct.ac_etime   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_etime), sizeof(acct.ac_etime));
    printf("acct.ac_mem     @%3lu sizeof %3lu\n", offsetof(struct acct, ac_mem), sizeof(acct.ac_mem));
    printf("acct.ac_io      @%3lu sizeof %3lu\n", offsetof(struct acct, ac_io), sizeof(acct.ac_io));
    printf("acct.ac_rw      @%3lu sizeof %3lu\n", offsetof(struct acct, ac_rw), sizeof(acct.ac_rw));
    printf("acct.ac_minflt  @%3lu sizeof %3lu\n", offsetof(struct acct, ac_minflt), sizeof(acct.ac_minflt));
    printf("acct.ac_majflt  @%3lu sizeof %3lu\n", offsetof(struct acct, ac_majflt), sizeof(acct.ac_majflt));
    printf("acct.ac_swaps   @%3lu sizeof %3lu\n", offsetof(struct acct, ac_swaps), sizeof(acct.ac_swaps));
    printf("acct.ac_ahz     @%3lu sizeof %3lu\n", offsetof(struct acct, ac_ahz), sizeof(acct.ac_ahz));
    printf("acct.ac_stat    @%3lu sizeof %3lu\n", offsetof(struct acct, ac_stat), sizeof(acct.ac_stat));
    printf("acct.ac_exitcode@%3lu sizeof %3lu\n", offsetof(struct acct, ac_exitcode), sizeof(acct.ac_exitcode));
    printf("acct.ac_comm    @%3lu sizeof %3lu\n", offsetof(struct acct, ac_comm), sizeof(acct.ac_comm));
    printf("acct.ac_amin    @%3lu sizeof %3lu\n", offsetof(struct acct, ac_amin), sizeof(acct.ac_amin));
    printf("acct.ac_cmin    @%3lu sizeof %3lu\n", offsetof(struct acct, ac_cmin), sizeof(acct.ac_cmin));
    printf("acct.etime_hi   @%3lu sizeof %3lu\n", offsetof(struct acct, etime_hi), sizeof(acct.etime_hi));
    printf("acct.etime_lo   @%3lu sizeof %3lu\n", offsetof(struct acct, etime_lo), sizeof(acct.etime_lo));
    printf("acct.ac_pad     @%3lu sizeof %3lu\n", offsetof(struct acct, ac_pad), sizeof(acct.ac_pad));
#endif

#if ACCT_V3_COMPILE
    printf("\n");
    printf("acct_v3                  sizeof %3lu\n", sizeof(acct_v3));
    printf("acct_v3.ac_flag     @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_flag), sizeof(acct_v3.ac_flag));
    printf("acct_v3.ac_version  @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_version), sizeof(acct_v3.ac_version));
    printf("acct_v3.ac_tty      @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_tty), sizeof(acct_v3.ac_tty));
    printf("acct_v3.ac_exitcode @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_exitcode), sizeof(acct_v3.ac_exitcode));
    printf("acct_v3.ac_uid      @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_uid), sizeof(acct_v3.ac_uid));
    printf("acct_v3.ac_gid      @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_gid), sizeof(acct_v3.ac_gid));
    printf("acct_v3.ac_pid      @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_pid), sizeof(acct_v3.ac_pid));
    printf("acct_v3.ac_ppid     @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_ppid), sizeof(acct_v3.ac_ppid));
    printf("acct_v3.ac_btime    @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_btime), sizeof(acct_v3.ac_btime));
    printf("acct_v3.ac_etime    @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_etime), sizeof(acct_v3.ac_etime));
    printf("acct_v3.ac_utime    @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_utime), sizeof(acct_v3.ac_utime));
    printf("acct_v3.ac_stime    @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_stime), sizeof(acct_v3.ac_stime));
    printf("acct_v3.ac_mem      @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_mem), sizeof(acct_v3.ac_mem));
    printf("acct_v3.ac_io       @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_io), sizeof(acct_v3.ac_io));
    printf("acct_v3.ac_rw       @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_rw), sizeof(acct_v3.ac_rw));
    printf("acct_v3.ac_amin     @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_amin), sizeof(acct_v3.ac_amin));
    printf("acct_v3.ac_cmin     @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_cmin), sizeof(acct_v3.ac_cmin));
    printf("acct_v3.ac_minflt   @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_minflt), sizeof(acct_v3.ac_minflt));
    printf("acct_v3.ac_majflt   @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_majflt), sizeof(acct_v3.ac_majflt));
    printf("acct_v3.ac_swaps    @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_swaps), sizeof(acct_v3.ac_swaps));
    printf("acct_v3.ac_comm     @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_comm), sizeof(acct_v3.ac_comm));
    printf("acct_v3.ac_pad      @%3lu sizeof %3lu\n", offsetof(struct acct_v3, ac_pad), sizeof(acct_v3.ac_pad));
#endif

#if FAILLOG_COMPILE
    // from faillog.h
    // https://github.com/shadow-maint/shadow/blob/4.8.1/lib/faillog.h
    printf("faillog                sizeof %3lu\n", sizeof(faillog));
    printf("faillog.fail_cnt  @%3lu sizeof %3lu\n", offsetof(struct faillog, fail_cnt), sizeof(faillog.fail_cnt));
    printf("faillog.fail_max  @%3lu sizeof %3lu\n", offsetof(struct faillog, fail_max), sizeof(faillog.fail_max));
    printf("faillog.fail_line @%3lu sizeof %3lu\n", offsetof(struct faillog, fail_line), sizeof(faillog.fail_line));
    printf("faillog.fail_time @%3lu sizeof %3lu\n", offsetof(struct faillog, fail_time), sizeof(faillog.fail_time));
    printf("faillog.fail_lock @%3lu sizeof %3lu\n", offsetof(struct faillog, fail_lock), sizeof(faillog.fail_lock));
    printf("faillog.fail_locktime @%3lu sizeof %3lu\n", offsetof(struct faillog, fail_locktime), sizeof(faillog.fail_locktime));
    printf("\n");
#endif

    #ifdef UT_NAMESIZE
    printf("UT_NAMESIZE %d\n", UT_NAMESIZE);
    #endif
    #ifdef UT_LINESIZE
    printf("UT_LINESIZE %d\n", UT_LINESIZE);
    #endif
    #ifdef UT_HOSTSIZE
    printf("UT_HOSTSIZE %d\n", UT_HOSTSIZE);
    #endif
    #ifdef UT_IDSIZE
    printf("UT_IDSIZE %d\n", UT_IDSIZE);
    #endif
    #ifdef UT_TIME_SIZE
    printf("UT_TIME_SIZE %d\n", UT_TIME_SIZE);
    #endif

    #ifdef LASTLOG_FILE
    printf("LASTLOG_FILE     '%s'\n", (LASTLOG_FILE));
    #endif
    #ifdef LASTLOG_FILENAME
    printf("LASTLOG_FILENAME '%s'\n", (LASTLOG_FILENAME));
    #endif
    #ifdef _PATH_LASTLOG
    printf("_PATH_LASTLOG    '%s'\n", (_PATH_LASTLOG));
    #endif

#if LASTLOG_COMPILE
    printf("lastlog               sizeof %3lu\n", sizeof(lastlog));
    printf("lastlog.ll_time  @%3lu sizeof %3lu\n", offsetof(struct lastlog, ll_time), sizeof(lastlog.ll_time));
    printf("lastlog.ll_line  @%3lu sizeof %3lu\n", offsetof(struct lastlog, ll_line), sizeof(lastlog.ll_line));
    printf("lastlog.ll_host  @%3lu sizeof %3lu\n", offsetof(struct lastlog, ll_host), sizeof(lastlog.ll_host));
    printf("\n");
#endif

    #ifdef LASTLOGX_FILE
    printf("LASTLOGX_FILE    '%s'\n", (LASTLOGX_FILE));
    #endif
    #ifdef LASTLOGX_FILENAME
    printf("LASTLOGX_FILENAME'%s'\n", (LASTLOGX_FILENAME));
    #endif
    #ifdef _PATH_LASTLOGX
    printf("_PATH_LASTLOGX   '%s'\n", (_PATH_LASTLOGX));
    #endif

#if LASTLOGX_COMPILE
    printf("\n");
    printf("lastlogx               sizeof %3lu\n", sizeof(lastlogx));
    printf("lastlogx.ll_tv    @%3lu sizeof %3lu\n", offsetof(struct lastlogx, ll_tv), sizeof(lastlogx.ll_tv));
    printf("lastlogx.ll_line  @%3lu sizeof %3lu\n", offsetof(struct lastlogx, ll_line), sizeof(lastlogx.ll_line));
    printf("lastlogx.ll_host  @%3lu sizeof %3lu\n", offsetof(struct lastlogx, ll_host), sizeof(lastlogx.ll_host));
    printf("lastlogx.ll_ss    @%3lu sizeof %3lu\n", offsetof(struct lastlogx, ll_ss), sizeof(lastlogx.ll_ss));
    printf("\n");
#endif

#if TIMEVAL_COMPILE
    printf("\n");
    printf("timeval               sizeof %3lu\n", sizeof(timeval));
    printf("timeval.tv_sec   @%3lu sizeof %3lu\n", offsetof(struct timeval, tv_sec), sizeof(timeval.tv_sec));
    printf("timeval.tv_usec  @%3lu sizeof %3lu\n", offsetof(struct timeval, tv_usec), sizeof(timeval.tv_usec));
    printf("\n");
#endif

#if __TIMEVAL_COMPILE
    printf("\n");
    printf("__timeval               sizeof %3lu\n", sizeof(__timeval));
    printf("__timeval.tv_sec   @%3lu sizeof %3lu\n", offsetof(struct __timeval, tv_sec), sizeof(__timeval.tv_sec));
    printf("__timeval.tv_usec  @%3lu sizeof %3lu\n", offsetof(struct __timeval, tv_usec), sizeof(__timeval.tv_usec));
    printf("\n");
#endif

    // Linux utmp.h
    #ifdef _HAVE_UT_TYPE
    printf("_HAVE_UT_TYPE\n");
    #endif
    #ifdef _HAVE_UT_PID
    printf("_HAVE_UT_PID\n");
    #endif
    #ifdef _HAVE_UT_ID
    printf("_HAVE_UT_ID\n");
    #endif
    #ifdef _HAVE_UT_TV
    printf("_HAVE_UT_TV\n");
    #endif

    // FreeBSD /usr/src/include/utmpx.h
    #ifdef _HAVE_UT_SESSION
    printf("_HAVE_UT_SESSION\n");
    #endif
    #ifdef _HAVE_UT_ADDR
    printf("_HAVE_UT_ADDR\n");
    #endif
    #ifdef _HAVE_UT_ADDR_V6
    printf("_HAVE_UT_ADDR_V6\n");
    #endif

    // utx
    // from FreeBSD /usr/src/lib/libc/gen/utxdb.h
    #ifdef _PATH_UTX_ACTIVE
    printf("_PATH_UTX_ACTIVE    '%s'\n", (_PATH_UTX_ACTIVE));
    #endif
    #ifdef _PATH_UTX_LASTLOGIN
    printf("_PATH_UTX_LASTLOGIN '%s'\n", (_PATH_UTX_LASTLOGIN));
    #endif
    #ifdef PATH_UTX_LOG
    printf("PATH_UTX_LOG        '%s'\n", (PATH_UTX_LOG));
    #endif
    #ifdef _PATH_UTX_LOG
    printf("_PATH_UTX_LOG       '%s'\n", (_PATH_UTX_LOG));
    #endif
    #ifdef _PATH_UTX_LOGX
    printf("_PATH_UTX_LOGX      '%s'\n", (_PATH_UTX_LOGX));
    #endif
    #ifdef _PATH_UTX_USERS
    printf("_PATH_UTX_USERS     '%s'\n", (_PATH_UTX_USERS));
    #endif
    // utmp
    #ifdef UTMP_FILE
    printf("UTMP_FILE         '%s'\n", (UTMP_FILE));
    #endif
    #ifdef UTMP_FILENAME
    printf("UTMP_FILENAME     '%s'\n", (UTMP_FILENAME));
    #endif
    #ifdef PATH_UTMP
    printf("PATH_UTMP         '%s'\n", (PATH_UTMP));
    #endif
    #ifdef _PATH_UTMP
    printf("_PATH_UTMP        '%s'\n", (_PATH_UTMP));
    #endif
    // wtmp
    #ifdef WTMP_FILE
    printf("WTMP_FILE         '%s'\n", (WTMP_FILE));
    #endif
    #ifdef WTMP_FILENAME
    printf("WTMP_FILENAME     '%s'\n", (WTMP_FILENAME));
    #endif
    #ifdef PATH_WTMP
    printf("PATH_WTMP         '%s'\n", (PATH_WTMP));
    #endif
    #ifdef _PATH_WTMP
    printf("_PATH_WTMP        '%s'\n", (_PATH_WTMP));
    #endif
    // btmp
    #ifdef BTMP_FILE
    printf("BTMP_FILE         '%s'\n", (BTMP_FILE));
    #endif
    #ifdef BTMP_FILENAME
    printf("BTMP_FILENAME     '%s'\n", (BTMP_FILENAME));
    #endif
    #ifdef PATH_BTMP
    printf("PATH_BTMP         '%s'\n", (PATH_BTMP));
    #endif
    #ifdef _PATH_BTMP
    printf("_PATH_BTMP        '%s'\n", (_PATH_BTMP));
    #endif

    // from utmp.h
    // https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmp.h
    #ifdef UT_NAMESIZE
    printf("UT_NAMESIZE %d\n", UT_NAMESIZE);
    #endif
    #ifdef UT_LINESIZE
    printf("UT_LINESIZE %d\n", UT_LINESIZE);
    #endif
    #ifdef UT_HOSTSIZE
    printf("UT_HOSTSIZE %d\n", UT_HOSTSIZE);
    #endif
    //
    #ifdef UT_IDSIZE
    printf("UT_IDSIZE %d\n", UT_IDSIZE);
    #endif
    #ifdef UT_TIME_SIZE
    printf("UT_TIME_SIZE %d\n", UT_TIME_SIZE);
    #endif
    #ifdef UT_TV
    printf("UT_TV\n");
    #endif
    #ifdef UT_ADDR
    printf("UT_ADDR\n");
    #endif
    #ifdef UT_ADDR_V6
    printf("UT_ADDR_V6\n");
    #endif

#if UTMP_COMPILE
    printf("\n");
    printf("utmp                   sizeof %3lu\n", sizeof(utmp));
    printf("utmp.ut_type      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_type), sizeof(utmp.ut_type));
    printf("utmp.ut_pid       @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_pid), sizeof(utmp.ut_pid));
    printf("utmp.ut_id        @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_id), sizeof(utmp.ut_id));
    printf("utmp.ut_line      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_line), sizeof(utmp.ut_line));
    printf("utmp.ut_user      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_user), sizeof(utmp.ut_user));
    printf("utmp.ut_name      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_name), sizeof(utmp.ut_name));
    printf("utmp.ut_host      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_host), sizeof(utmp.ut_host));
    printf("utmp.ut_exit      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_exit), sizeof(utmp.ut_exit));
    printf("utmp.ut_session   @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_session), sizeof(utmp.ut_session));
    printf("utmp.ut_time      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_time), sizeof(utmp.ut_time));
    printf("utmp.ut_xtime     @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_xtime), sizeof(utmp.ut_xtime));
    printf("utmp.ut_tv        @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_tv), sizeof(utmp.ut_tv));
    printf("utmp.ut_tv.tv_sec @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_tv.tv_sec), sizeof(utmp.ut_tv.tv_sec));
    printf("utmp.ut_tv.tv_usec@%3lu sizeof %3lu\n", offsetof(struct utmp, ut_tv.tv_usec), sizeof(utmp.ut_tv.tv_usec));
    printf("utmp.ut_addr      @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_addr), sizeof(utmp.ut_addr));
    printf("utmp.ut_addr_v6   @%3lu sizeof %3lu\n", offsetof(struct utmp, ut_addr_v6), sizeof(utmp.ut_addr_v6));
    printf("\n");
#endif

    // utmpx
    #ifdef UTMPX_FILE
    printf("UTMPX_FILE         '%s'\n", (UTMPX_FILE));
    #endif
    #ifdef UTMPX_FILENAME
    printf("UTMPX_FILENAME     '%s'\n", (UTMPX_FILENAME));
    #endif
    #ifdef PATH_UTMPX
    printf("PATH_UTMPX         '%s'\n", (PATH_UTMPX));
    #endif
    #ifdef _PATH_UTMPX
    printf("_PATH_UTMPX        '%s'\n", (_PATH_UTMPX));
    #endif
    // wtmpx
    #ifdef WTMPX_FILE
    printf("WTMPX_FILE         '%s'\n", (WTMPX_FILE));
    #endif
    #ifdef WTMPX_FILENAME
    printf("WTMPX_FILENAME     '%s'\n", (WTMPX_FILENAME));
    #endif
    #ifdef PATH_WTMPX
    printf("PATH_WTMPX        '%s'\n", (PATH_WTMPX));
    #endif
    #ifdef _PATH_WTMPX
    printf("_PATH_WTMPX       '%s'\n", (_PATH_WTMPX));
    #endif
    // btmpx
    #ifdef BTMPX_FILE
    printf("BTMPX_FILE         '%s'\n", (BTMPX_FILE));
    #endif
    #ifdef BTMPX_FILENAME
    printf("BTMPX_FILENAME     '%s'\n", (BTMPX_FILENAME));
    #endif
    #ifdef PATH_BTMPX
    printf("PATH_BTMPX         '%s'\n", (PATH_BTMPX));
    #endif
    #ifdef _PATH_BTMPX
    printf("_PATH_BTMPX        '%s'\n", (_PATH_BTMPX));
    #endif

    // from utmpx.h
    // https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmpx.h
    #ifdef EMPTY
    printf("EMPTY %d\n", EMPTY);
    #endif
    #ifdef RUN_LVL
    printf("RUN_LVL %d\n", RUN_LVL);
    #endif
    #ifdef BOOT_TIME
    printf("BOOT_TIME %d\n", BOOT_TIME);
    #endif
    #ifdef OLD_TIME
    printf("OLD_TIME %d\n", OLD_TIME);
    #endif
    #ifdef NEW_TIME
    printf("NEW_TIME %d\n", NEW_TIME);
    #endif
    #ifdef INIT_PROCESS
    printf("INIT_PROCESS %d\n", INIT_PROCESS);
    #endif
    #ifdef LOGIN_PROCESS
    printf("LOGIN_PROCESS %d\n", LOGIN_PROCESS);
    #endif
    #ifdef USER_PROCESS
    printf("USER_PROCESS %d\n", USER_PROCESS);
    #endif
    #ifdef DEAD_PROCESS
    printf("DEAD_PROCESS %d\n", DEAD_PROCESS);
    #endif
    #ifdef ACCOUNTING
    printf("ACCOUNTING %d\n", ACCOUNTING);
    #endif
    #ifdef SIGNATURE
    printf("SIGNATURE %d\n", SIGNATURE);
    #endif
    #ifdef DOWN_TIME
    printf("DOWN_TIME %d\n", DOWN_TIME);
    #endif

    #ifdef UTX_USERSIZE
    printf("UTX_USERSIZE %d\n", UTX_USERSIZE);
    #endif
    #ifdef _UTX_USERSIZE
    printf("_UTX_USERSIZE %d\n", _UTX_USERSIZE);
    #endif
    #ifdef UTX_LINESIZE
    printf("UTX_LINESIZE %d\n", UTX_LINESIZE);
    #endif
    #ifdef _UTX_LINESIZE
    printf("_UTX_LINESIZE %d\n", _UTX_LINESIZE);
    #endif
    #ifdef UTX_IDSIZE
    printf("UTX_IDSIZE %d\n", UTX_IDSIZE);
    #endif
    #ifdef _UTX_IDSIZE
    printf("_UTX_IDSIZE %d\n", _UTX_IDSIZE);
    #endif
    #ifdef UTX_HOSTSIZE
    printf("UTX_HOSTSIZE %d\n", UTX_HOSTSIZE);
    #endif
    #ifdef _UTX_HOSTSIZE
    printf("_UTX_HOSTSIZE %d\n", _UTX_HOSTSIZE);
    #endif
    #ifdef UTX_TIME_SIZE
    printf("UTX_TIME_SIZE %d\n", UTX_TIME_SIZE);
    #endif
    #ifdef UTX_TV
    printf("UTX_TV\n");
    #endif
    #ifdef UTX_ADDR
    printf("UTX_ADDR\n");
    #endif
    #ifdef UTX_ADDR_V6
    printf("UTX_ADDR_V6\n");
    #endif

    #ifdef _UTX_PADSIZE
    printf("_UTX_PADSIZE %d\n", _UTX_PADSIZE);
    #endif

#if UTMPX_COMPILE
    printf("\n");
    printf("utmpx                   sizeof %3lu\n", sizeof(utmpx));
    printf("utmpx.ut_type      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_type), sizeof(utmpx.ut_type));
    printf("utmpx.ut_pid       @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_pid), sizeof(utmpx.ut_pid));
    printf("utmpx.ut_line      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_line), sizeof(utmpx.ut_line));
    printf("utmpx.ut_id        @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_id), sizeof(utmpx.ut_id));
    printf("utmpx.ut_user      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_user), sizeof(utmpx.ut_user));
    printf("utmpx.ut_name      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_name), sizeof(utmpx.ut_name));
    printf("utmpx.ut_host      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_host), sizeof(utmpx.ut_host));
    printf("utmpx.ut_exit      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_exit), sizeof(utmpx.ut_exit));
    printf("utmpx.ut_ss        @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_ss), sizeof(utmpx.ut_ss));
    printf("utmpx.ut_session   @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_session), sizeof(utmpx.ut_session));
    printf("utmpx.ut_time      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_time), sizeof(utmpx.ut_time));
    printf("utmpx.ut_xtime     @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_xtime), sizeof(utmpx.ut_xtime));
    printf("utmpx.ut_tv        @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_tv), sizeof(utmpx.ut_tv));
    printf("utmpx.ut_tv.tv_sec @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_tv.tv_sec), sizeof(utmpx.ut_tv.tv_sec));
    printf("utmpx.ut_tv.tv_usec@%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_tv.tv_usec), sizeof(utmpx.ut_tv.tv_usec));
    printf("utmpx.ut_addr      @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_addr), sizeof(utmpx.ut_addr));
    printf("utmpx.ut_addr_v6   @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_addr_v6), sizeof(utmpx.ut_addr_v6));
    printf("utmpx.ut_pad       @%3lu sizeof %3lu\n", offsetof(struct utmpx, ut_pad), sizeof(utmpx.ut_pad));
    printf("\n");
#endif

#if UTXDB_COMPILE
    printf("\n");
    printf("utxdb                 sizeof %3lu\n", sizeof(utxdb));
    printf("utxdb.ut_type    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_type), sizeof(utxdb.ut_type));
    printf("utxdb.ut_pid     @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_pid), sizeof(utxdb.ut_pid));
    printf("utxdb.ut_line    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_line), sizeof(utxdb.ut_line));
    printf("utxdb.ut_id      @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_id), sizeof(utxdb.ut_id));
    printf("utxdb.ut_user    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_user), sizeof(utxdb.ut_user));
    printf("utxdb.ut_name    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_name), sizeof(utxdb.ut_name));
    printf("utxdb.ut_host    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_host), sizeof(utxdb.ut_host));
    printf("utxdb.ut_exit    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_exit), sizeof(utxdb.ut_exit));
    printf("utxdb.ut_session @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_session), sizeof(utxdb.ut_session));
    printf("utxdb.ut_time    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_time), sizeof(utxdb.ut_time));
    printf("utxdb.ut_xtime   @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_xtime), sizeof(utxdb.ut_xtime));
    printf("utxdb.ut_tv      @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_tv), sizeof(utxdb.ut_tv));
    printf("utxdb.ut_tv.tv_sec  @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_tv.tv_sec), sizeof(utxdb.ut_tv.tv_sec));
    printf("utxdb.ut_tv.tv_usec @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_tv.tv_usec), sizeof(utxdb.ut_tv.tv_usec));
    printf("utxdb.ut_addr    @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_addr), sizeof(utxdb.ut_addr));
    printf("utxdb.ut_addr_v6 @%3lu sizeof %3lu\n", offsetof(struct utxdb, ut_addr_v6), sizeof(utxdb.ut_addr_v6));
    printf("\n");
#endif

#if FUTX_COMPILE
    printf("\n");
    printf("futx                 sizeof %3lu\n", sizeof(futx));
    printf("futx.fu_type    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_type), sizeof(futx.fu_type));
    printf("futx.fu_tv      @%3lu sizeof %3lu\n", offsetof(struct futx, fu_tv), sizeof(futx.fu_tv));
    printf("futx.fu_id      @%3lu sizeof %3lu\n", offsetof(struct futx, fu_id), sizeof(futx.fu_id));
    printf("futx.fu_pid     @%3lu sizeof %3lu\n", offsetof(struct futx, fu_pid), sizeof(futx.fu_pid));
    printf("futx.fu_user    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_user), sizeof(futx.fu_user));
    printf("futx.fu_line    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_line), sizeof(futx.fu_line));
    printf("futx.fu_host    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_host), sizeof(futx.fu_host));
    printf("futx.fu_time    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_time), sizeof(futx.fu_time));
    printf("futx.fu_exit    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_exit), sizeof(futx.fu_exit));
    printf("futx.fu_session @%3lu sizeof %3lu\n", offsetof(struct futx, fu_session), sizeof(futx.fu_session));
    printf("futx.fu_addr    @%3lu sizeof %3lu\n", offsetof(struct futx, fu_addr), sizeof(futx.fu_addr));
    printf("futx.fu_addr_v6 @%3lu sizeof %3lu\n", offsetof(struct futx, fu_addr_v6), sizeof(futx.fu_addr_v6));
    printf("\n");
#endif

    return 0;
}
