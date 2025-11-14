/*
 * This file is part of the Trezor project, https://trezor.io/
 *
 * Copyright (c) SatoshiLabs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#pragma once

#include <sys/bootutils.h>

// Shows an error message and shuts down the device.
//
// If the title is NULL, it will be set to "INTERNAL ERROR".
// If the message is NULL, it will be ignored.
// If the footer is NULL, it will be set to "PLEASE VISIT TREZOR.IO/RSOD".
void __attribute__((noreturn))
error_shutdown_ex(const char *title, const char *message, const char *footer);

// Shows an error message and shuts down the device.
//
// Same as `error_shutdown_ex()` but with a default header and footer.
void __attribute__((noreturn)) error_shutdown(const char *message);

// Do not use this function directly, use the `ensure()` macro instead.
void __attribute__((noreturn))
__fatal_error(const char *msg, const char *file, int line);

// Checks for an expression and if it is false, shows an error message
// and shuts down the device.
#define ensure(expr, msg) \
  (((expr) == sectrue) ? (void)0 : __fatal_error(msg, __FILE_NAME__, __LINE__))

// Shows WIPE CODE ENTERED screeen and shuts down the device.
void __attribute__((noreturn)) show_wipe_code_screen(void);

// Shows TOO MANY PIN ATTEMPTS screen and shuts down the device.
void __attribute__((noreturn)) show_pin_too_many_screen(void);

// Shows INSTALL RESTRICTED screen and shuts down the device.
void __attribute__((noreturn)) show_install_restricted_screen(void);

// Shows wipe information screen
void show_wipe_info(const bootutils_wipe_info_t *info);


#include <errno.h>


typedef struct {
  // Do not access this field directly,
  // use `ts_ok()` and `ts_error()` macros.
  uint32_t code;
} ts_t;

#define ts_wur_t \
 ts_t __attribute__((warn_unused_result))


#define TS_BUILD(code) ((const ts_t){(code)})

// Error status codes
#define TS_OK TS_BUILD(0)
#define TS_EPERM TS_BUILD(EPERM)
#define TS_ENOENT TS_BUILD(ENOENT)
#define TS_EIO TS_BUILD(EIO)

// Extracts the status code integer value.
#define ts_code(status) ((status).code)

// Returns `true` if status code is `TS_OK`
#define ts_ok(status) (ts_code(status) == ts_code(TS_OK))

// Returns `true` if status code is NOT `TS_OK`
#define ts_error(status) (ts_code(status) != ts_code(TS_OK))

// Returns `true` if both status codes are equal
#define ts_eq(status1, status2) (ts_code(status1) == ts_code(status2))



// Ensures that status code is `TS_OK`.
// If not, it shows an error message and shuts down the device.
#define ensure_ok(status, msg)                     \
  do {                                             \
    if (!ts_ok(status)) {                          \
      __fatal_error(msg, __FILE_NAME__, __LINE__); \
    }                                              \
  } while (0)

// Ensures that condition is evaluated as `true`.
// If not, it shows an error message and shuts down the device.
#define ensure_true(cond, msg)                     \
  do {                                             \
    if (!(cond)) {                                 \
      __fatal_error(msg, __FILE_NAME__, __LINE__); \
    }                                              \
  } while (0)

// Ensures that condition is evaluated as `sectrue`.
// If not, it shows an error message and shuts down the device.
#define ensure(seccond, msg)                       \
  do {                                             \
    if ((seccond) != sectrue) {                    \
      __fatal_error(msg, __FILE_NAME__, __LINE__); \
    }                                              \
  } while (0)



-#define	EPERM		1		/* Operation not permitted */
+#define	ENOENT		2		/* No such file or directory */
-#define	ESRCH		3		/* No such process */
-#define	EINTR		4		/* Interrupted system call */
+#define	EIO		5		/* Input/output error */
+#define	ENXIO		6		/* Device not configured */
+#define	E2BIG		7		/* Argument list too long */
-#define	ENOEXEC		8		/* Exec format error */
-#define	EBADF		9		/* Bad file descriptor */
-#define	ECHILD		10		/* No child processes */
-#define	EDEADLK		11		/* Resource deadlock avoided */
					/* 11 was EAGAIN */
+#define	ENOMEM		12		/* Cannot allocate memory */
+#define	EACCES		13		/* Permission denied */
+#define	EFAULT		14		/* Bad address */
#ifndef _POSIX_SOURCE
-#define	ENOTBLK		15		/* Block device required */
#endif
+#define	EBUSY		16		/* Device busy */
+#define	EEXIST		17		/* File exists */
-#define	EXDEV		18		/* Cross-device link */
+#define	ENODEV		19		/* Operation not supported by device */
-#define	ENOTDIR		20		/* Not a directory */
-#define	EISDIR		21		/* Is a directory */
+#define	EINVAL		22		/* Invalid argument */
-#define	ENFILE		23		/* Too many open files in system */
-#define	EMFILE		24		/* Too many open files */
-#define	ENOTTY		25		/* Inappropriate ioctl for device */
#ifndef _POSIX_SOURCE
-#define	ETXTBSY		26		/* Text file busy */
#endif
-#define	EFBIG		27		/* File too large */
+#define	ENOSPC		28		/* No space left on device */
-#define	ESPIPE		29		/* Illegal seek */
-#define	EROFS		30		/* Read-only filesystem */
-#define	EMLINK		31		/* Too many links */
-#define	EPIPE		32		/* Broken pipe */
/* math software */
-#define	EDOM		33		/* Numerical argument out of domain */
+#define	ERANGE		34		/* Result too large */
/* non-blocking and interrupt i/o */
+#define	EAGAIN		35		/* Resource temporarily unavailable */
#ifndef _POSIX_SOURCE
+#define	EWOULDBLOCK	EAGAIN		/* Operation would block */
+#define	EINPROGRESS	36		/* Operation now in progress */
+#define	EALREADY	37		/* Operation already in progress */
/* ipc/network software -- argument errors */
-#define	ENOTSOCK	38		/* Socket operation on non-socket */
-#define	EDESTADDRREQ	39		/* Destination address required */
+#define	EMSGSIZE	40		/* Message too long */
-#define	EPROTOTYPE	41		/* Protocol wrong type for socket */
-#define	ENOPROTOOPT	42		/* Protocol not available */
#define	EPROTONOSUPPORT	43		/* Protocol not supported */
#define	ESOCKTNOSUPPORT	44		/* Socket type not supported */
#define	EOPNOTSUPP	45		/* Operation not supported */
#define	ENOTSUP		EOPNOTSUPP	/* Operation not supported */
#define	EPFNOSUPPORT	46		/* Protocol family not supported */
#define	EAFNOSUPPORT	47		/* Address family not supported by protocol family */
#define	EADDRINUSE	48		/* Address already in use */
#define	EADDRNOTAVAIL	49		/* Can't assign requested address */
/* ipc/network software -- operational errors */
#define	ENETDOWN	50		/* Network is down */
#define	ENETUNREACH	51		/* Network is unreachable */
#define	ENETRESET	52		/* Network dropped connection on reset */
+#define	ECONNABORTED	53		/* Software caused connection abort */
#define	ECONNRESET	54		/* Connection reset by peer */
#define	ENOBUFS		55		/* No buffer space available */
#define	EISCONN		56		/* Socket is already connected */
#define	ENOTCONN	57		/* Socket is not connected */
#define	ESHUTDOWN	58		/* Can't send after socket shutdown */
#define	ETOOMANYREFS	59		/* Too many references: can't splice */
+#define	ETIMEDOUT	60		/* Operation timed out */
#define	ECONNREFUSED	61		/* Connection refused */
#define	ELOOP		62		/* Too many levels of symbolic links */
#endif /* _POSIX_SOURCE */
#define	ENAMETOOLONG	63		/* File name too long */
/* should be rearranged */
#ifndef _POSIX_SOURCE
#define	EHOSTDOWN	64		/* Host is down */
#define	EHOSTUNREACH	65		/* No route to host */
#endif /* _POSIX_SOURCE */
#define	ENOTEMPTY	66		/* Directory not empty */
/* quotas & mush */
#ifndef _POSIX_SOURCE
#define	EPROCLIM	67		/* Too many processes */
#define	EUSERS		68		/* Too many users */
#define	EDQUOT		69		/* Disc quota exceeded */
/* Network File System */
#define	ESTALE		70		/* Stale NFS file handle */
#define	EREMOTE		71		/* Too many levels of remote in path */
#define	EBADRPC		72		/* RPC struct is bad */
#define	ERPCMISMATCH	73		/* RPC version wrong */
#define	EPROGUNAVAIL	74		/* RPC prog. not avail */
#define	EPROGMISMATCH	75		/* Program version wrong */
#define	EPROCUNAVAIL	76		/* Bad procedure for program */
#endif /* _POSIX_SOURCE */
#define	ENOLCK		77		/* No locks available */
#define	ENOSYS		78		/* Function not implemented */
#ifndef _POSIX_SOURCE
#define	EFTYPE		79		/* Inappropriate file type or format */
+#define	EAUTH		80		/* Authentication error */
#define	ENEEDAUTH	81		/* Need authenticator */
#define	EIDRM		82		/* Identifier removed */
#define	ENOMSG		83		/* No message of desired type */
#define	EOVERFLOW	84		/* Value too large to be stored in data type */
#define	ECANCELED	85		/* Operation canceled */
#define	EILSEQ		86		/* Illegal byte sequence */
#define	ENOATTR		87		/* Attribute not found */
#define	EDOOFUS		88		/* Programming error */
#endif /* _POSIX_SOURCE */
#define	EBADMSG		89		/* Bad message */
#define	EMULTIHOP	90		/* Multihop attempted */
#define	ENOLINK		91		/* Link has been severed */
#define	EPROTO		92		/* Protocol error */
#ifndef _POSIX_SOURCE
#define	ENOTCAPABLE	93		/* Capabilities insufficient */
#define	ECAPMODE	94		/* Not permitted in capability mode */
#endif /* _POSIX_SOURCE */
#ifndef _POSIX_SOURCE
#define	ELAST		94		/* Must be equal largest errno */
#endif /* _POSIX_SOURCE */
#ifdef _KERNEL
/* pseudo-errors returned inside kernel to modify return to process */
#define	ERESTART	(-1)		/* restart syscall */
#define	EJUSTRETURN	(-2)		/* don't modify regs, just return */
#define	ENOIOCTL	(-3)		/* ioctl not handled by this layer */
#define	EDIRIOCTL	(-4)		/* do direct ioctl in GEOM */
#endif