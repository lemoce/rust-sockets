#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <errno.h>
#include <stdarg.h>		/* ANSI C header file */
#include <syslog.h>		/* for syslog() */

#include <sys/types.h>
#include <sys/socket.h>

#include <netinet/in.h>	/* sockaddr_in{} and other Internet defns */
#include <netinet/tcp.h>

#include <linux/udp.h>
#include <linux/in.h>

#include <arpa/inet.h>	/* inet(3) functions */

#define	MAXLINE		4096	/* max text line length */
#define	BUFFSIZE	8192	/* buffer size for reads and writes */

#define SO_USELOOPBACK 0x0040

union val {
  int i_val;
  long l_val;
  struct linger linger_val;
  struct timeval timeval_val;
} val;

static char * sock_str_flag(union val *ptr, int len);
static char * sock_str_int(union val *ptr, int len);
static char * sock_str_linger(union val *ptr, int len);
static char * sock_str_timeval(union val *ptr, int len);


struct sock_opts {
  const char *opt_str;
  int opt_level;
  int opt_name;
  char *(*opt_val_str) (union val *, int);
} sock_opts[] =
  {
   { "SO_BROADCAST",          SOL_SOCKET,       SO_BROADCAST,           sock_str_flag },
   { "SO_DEBUG",              SOL_SOCKET,       SO_DEBUG,               sock_str_flag },
   { "SO_DONTROUTE",          SOL_SOCKET,       SO_DONTROUTE,           sock_str_flag },
   { "SO_ERROR",              SOL_SOCKET,       SO_ERROR,               sock_str_int },
   { "SO_KEEPALIVE",          SOL_SOCKET,       SO_KEEPALIVE,           sock_str_flag },
   { "SO_LINGER",             SOL_SOCKET,       SO_LINGER,              sock_str_linger },
   { "SO_OOBINLINE",          SOL_SOCKET,       SO_OOBINLINE,           sock_str_flag },
   { "SO_RCVBUF",             SOL_SOCKET,       SO_RCVBUF,              sock_str_int },
   { "SO_SNDBUF",             SOL_SOCKET,       SO_SNDBUF,              sock_str_int },
   { "SO_RCVLOWAT",           SOL_SOCKET,       SO_RCVLOWAT,            sock_str_int },
   { "SO_SNDLOWAT",           SOL_SOCKET,       SO_SNDLOWAT,            sock_str_int },
   { "SO_RCVTIMEO",           SOL_SOCKET,       SO_RCVTIMEO,            sock_str_timeval },
   { "SO_SNDTIMEO",           SOL_SOCKET,       SO_SNDTIMEO,            sock_str_timeval },
   { "SO_REUSEADDR",          SOL_SOCKET,       SO_REUSEADDR,           sock_str_flag },
#ifdef	SO_REUSEPORT
   { "SO_REUSEPORT",          SOL_SOCKET,       SO_REUSEPORT,           sock_str_flag },
#else
   { "SO_REUSEPORT",          0,                0,                      NULL },
#endif
   { "SO_TYPE",               SOL_SOCKET,       SO_TYPE,                sock_str_int },
   { "SO_USELOOPBACK",        SOL_SOCKET,       SO_USELOOPBACK,         sock_str_flag },
   { "IP_TOS",                IPPROTO_IP,       IP_TOS,                 sock_str_int },
   { "IP_TTL",                IPPROTO_IP,       IP_TTL,                 sock_str_int },
#ifdef	IPV6_DONTFRAG
   { "IPV6_DONTFRAG",         IPPROTO_IPV6,     IPV6_DONTFRAG,          sock_str_flag },
#else
   { "IPV6_DONTFRAG",         0,                0,                      NULL },
#endif
#ifdef	IPV6_UNICAST_HOPS
   { "IPV6_UNICAST_HOPS",     IPPROTO_IPV6,     IPV6_UNICAST_HOPS,      sock_str_int },
#else
   { "IPV6_UNICAST_HOPS",     0,                0,                      NULL },
#endif
#ifdef	IPV6_V6ONLY
   { "IPV6_V6ONLY",           IPPROTO_IPV6,     IPV6_V6ONLY,            sock_str_flag },
#else
   { "IPV6_V6ONLY",           0,                0,                      NULL },
#endif
   { "TCP_MAXSEG",            IPPROTO_TCP,      TCP_MAXSEG,             sock_str_int },
   { "TCP_NODELAY",           IPPROTO_TCP,      TCP_NODELAY,            sock_str_flag },
#ifdef	SCTP_AUTOCLOSE
   { "SCTP_AUTOCLOSE",        IPPROTO_SCTP,     SCTP_AUTOCLOSE,         sock_str_int },
#else
   { "SCTP_AUTOCLOSE",        0,                0,                      NULL },
#endif
#ifdef	SCTP_MAXBURST
   { "SCTP_MAXBURST",         IPPROTO_SCTP,     SCTP_MAXBURST,          sock_str_int },
#else
   { "SCTP_MAXBURST",         0,                0,                      NULL },
#endif
#ifdef	SCTP_MAXSEG
   { "SCTP_MAXSEG",           IPPROTO_SCTP,     SCTP_MAXSEG,            sock_str_int },
#else
   { "SCTP_MAXSEG",           0,                0,                      NULL },
#endif
#ifdef	SCTP_NODELAY
   { "SCTP_NODELAY",          IPPROTO_SCTP,     SCTP_NODELAY,           sock_str_flag },
#else
   { "SCTP_NODELAY",          0,                0,                      NULL },
#endif
   {NULL,                     0,                0,                      NULL }
  };

int daemon_proc;		/* set nonzero by daemon_init() */
static void err_doit(int, int, const char *, va_list);

void err_ret(const char *fmt, ...)
{
  va_list ap;

  va_start(ap, fmt);
  err_doit(1, LOG_INFO, fmt, ap);
  va_end(ap);
  return;
}

void err_quit(const char *fmt, ...)
{
  va_list ap;

  va_start(ap, fmt);
  err_doit(0, LOG_ERR, fmt, ap);
  va_end(ap);
  exit(1);
}

void err_sys(const char *fmt, ...)
{
  va_list ap;

  va_start(ap, fmt);
  err_doit(1, LOG_ERR, fmt, ap);
  va_end(ap);
  exit(1);
}

static void err_doit(int errnoflag, int level, const char *fmt, va_list ap)
{
  int errno_save, n;
  char buf[MAXLINE + 1];

  errno_save = errno;		/* value caller might want printed */
#ifdef	HAVE_VSNPRINTF
  vsnprintf(buf, MAXLINE, fmt, ap);	/* safe */
#else
  vsprintf(buf, fmt, ap);					/* not safe */
#endif
  n = strlen(buf);
  if (errnoflag)
    snprintf(buf + n, MAXLINE - n, ": %s", strerror(errno_save));
  strcat(buf, "\n");
  
  if (daemon_proc) {
    syslog(level, buf);
  } else {
    fflush(stdout);		/* in case stdout and stderr are the same */
    fputs(buf, stderr);
    fflush(stderr);
  }
  return;
}

int Socket(int family, int type, int protocol)
{
  int n;

  if ( (n = socket(family, type, protocol)) < 0)
    err_sys("socket error");
  return(n);
}

int main(int argc, char **argv)
{
  int fd;
  socklen_t len;
  struct sock_opts *ptr;

  for (ptr = sock_opts; ptr->opt_str != NULL; ptr++) {
    printf("%s: ", ptr->opt_str);
    if (ptr->opt_val_str == NULL)
      printf("(undefined)\n");
    else {
      switch(ptr->opt_level) {
      case SOL_SOCKET:
      case IPPROTO_IP:
      case IPPROTO_TCP:
        fd = Socket(AF_INET, SOCK_STREAM, 0);
        break;
#ifdef	IPV6
      case IPPROTO_IPV6:
        fd = Socket(AF_INET6, SOCK_STREAM, 0);
        break;
#endif
#ifdef	IPPROTO_SCTP
      case IPPROTO_SCTP:
        fd = Socket(AF_INET, SOCK_SEQPACKET, IPPROTO_SCTP);
        break;
#endif
      default:
        err_quit("Can't create fd for level %d\n", ptr->opt_level);
      }
      
      len = sizeof(val);
      if (getsockopt(fd, ptr->opt_level, ptr->opt_name,
                     &val, &len) == -1) {
        err_ret("getsockopt error");
      } else {
        printf("default = %s\n", (*ptr->opt_val_str)(&val, len));
      }
      close(fd);
    }
  }
  exit(0);
}

static char strres[128];

static char * sock_str_flag(union val *ptr, int len)
{
  if (len != sizeof(int))
    snprintf(strres, sizeof(strres), "size (%d) not sizeof(int)", len);
  else
    snprintf(strres, sizeof(strres), "%s", (ptr->i_val == 0)? "off": "on");
  return(strres);
}

static char * sock_str_int(union val *ptr, int len)
{
  if (len != sizeof(int))
    snprintf(strres, sizeof(strres), "size (%d) not sizeof(int)", len);
  else
    snprintf(strres, sizeof(strres), "%d", ptr->i_val);
  return(strres);
}

static char * sock_str_linger(union val *ptr, int len)
{
  struct linger	*lptr = &ptr->linger_val;
  
  if (len != sizeof(struct linger))
    snprintf(strres, sizeof(strres),
             "size (%d) not sizeof(struct linger)", len);
  else
    snprintf(strres, sizeof(strres), "l_onoff = %d, l_linger = %d",
             lptr->l_onoff, lptr->l_linger);
  return(strres);
}

static char * sock_str_timeval(union val *ptr, int len)
{
  struct timeval *tvptr = &ptr->timeval_val;

  if (len != sizeof(struct timeval))
    snprintf(strres, sizeof(strres),
             "size (%d) not sizeof(struct timeval)", len);
  else
    snprintf(strres, sizeof(strres), "%ld sec, %ld usec",
             tvptr->tv_sec, tvptr->tv_usec);
  return(strres);
}

