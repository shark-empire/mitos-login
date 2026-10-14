/* Prints the values that src/sys/ffi.rs hard-codes, straight from the system headers. */
#define _GNU_SOURCE
#include <stdio.h>
#include <stddef.h>
#include <errno.h>
#include <poll.h>
#include <signal.h>
#include <sys/signalfd.h>
#include <sys/socket.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <sys/reboot.h>
#include <fcntl.h>
#include <dlfcn.h>
#include <termios.h>
#include <pwd.h>
#include <linux/vt.h>
#include <linux/kd.h>
#include <linux/fb.h>
#include <linux/input.h>
#include <sys/wait.h>

#define C(n) printf("%s=%ld\n", #n, (long)(n))
#define U(n) printf("%s=%lu\n", #n, (unsigned long)(n))
#define SZ(t) printf("sizeof_%s=%zu\n", #t, sizeof(t))
#define OFF(t,f) printf("offsetof_%s_%s=%zu\n", #t, #f, offsetof(t,f))

int main(void) {
  C(EPERM); C(ENOENT); C(ESRCH); C(EINTR); C(ECHILD); C(EBADF); C(EAGAIN); C(ERANGE);
  C(POLLIN); C(POLLOUT); C(POLLERR); C(POLLHUP); C(POLLNVAL);
  C(SIG_BLOCK); C(SIG_SETMASK); C(SFD_NONBLOCK); C(SFD_CLOEXEC); C(SIGKILL); C(SIGHUP); C(SIGINT); C(SIGQUIT); C(SIGUSR1); C(SIGUSR2); C(SIGTERM); C(SIGCHLD);
  C(SOL_SOCKET); C(SO_PEERCRED);
  C(PR_SET_PDEATHSIG); C(PR_SET_DUMPABLE); C(PR_SET_NO_NEW_PRIVS); C(RLIMIT_CORE);
  C(F_DUPFD_CLOEXEC); C(O_RDWR); C(O_NOCTTY); U(TIOCSCTTY);
  C(PROT_READ); C(PROT_WRITE); C(MAP_SHARED);
  C(RTLD_NOW);
  U(VT_ACTIVATE); U(VT_WAITACTIVE); U(VT_LOCKSWITCH); U(VT_UNLOCKSWITCH);
  U(KDSETMODE); U(KDSKBMODE); U(KDGKBMODE);
  U(FBIOGET_VSCREENINFO); U(FBIOGET_FSCREENINFO);

  U(EVIOCGRAB);
  U(VT_OPENQRY); U(VT_GETSTATE);
  SZ(struct vt_stat); OFF(struct vt_stat, v_active); OFF(struct vt_stat, v_signal); OFF(struct vt_stat, v_state);
  printf("EVIOCGNAME_32=%lu\n", (unsigned long)EVIOCGNAME(32));
  printf("EVIOCGNAME_256=%lu\n", (unsigned long)EVIOCGNAME(256));
  printf("EVIOCGBIT_0_32=%lu\n", (unsigned long)EVIOCGBIT(0,32));
  printf("EVIOCGBIT_EVKEY_96=%lu\n", (unsigned long)EVIOCGBIT(EV_KEY,96));
  printf("EVIOCGBIT_EVREL_8=%lu\n", (unsigned long)EVIOCGBIT(EV_REL,8));
  U(ECHO); C(TCSANOW);
  U(RB_AUTOBOOT); U(RB_POWER_OFF); C(WNOHANG);
  SZ(struct pollfd); SZ(sigset_t); SZ(struct rlimit); SZ(struct ucred); SZ(struct passwd);
  SZ(struct fb_var_screeninfo); SZ(struct fb_fix_screeninfo); SZ(struct termios); SZ(struct signalfd_siginfo);
  SZ(struct input_event);
  OFF(struct termios, c_lflag);
  OFF(struct passwd, pw_uid); OFF(struct passwd, pw_gecos); OFF(struct passwd, pw_dir); OFF(struct passwd, pw_shell);
  OFF(struct fb_var_screeninfo, bits_per_pixel); OFF(struct fb_var_screeninfo, red); OFF(struct fb_var_screeninfo, height);
  OFF(struct fb_var_screeninfo, width); OFF(struct fb_var_screeninfo, reserved);
  OFF(struct fb_fix_screeninfo, smem_len); OFF(struct fb_fix_screeninfo, line_length); OFF(struct fb_fix_screeninfo, mmio_start);
  OFF(struct input_event, type); OFF(struct input_event, code); OFF(struct input_event, value);
  /* input event codes used by seats/input.rs */
  C(EV_SYN); C(EV_KEY); C(EV_REL); C(REL_X); C(REL_Y); C(REL_WHEEL);
  C(BTN_LEFT); C(BTN_RIGHT); C(BTN_MIDDLE);
  C(KEY_ESC); C(KEY_BACKSPACE); C(KEY_TAB); C(KEY_ENTER); C(KEY_KPENTER); C(KEY_SPACE);
  C(KEY_UP); C(KEY_DOWN); C(KEY_LEFT); C(KEY_RIGHT); C(KEY_HOME); C(KEY_END); C(KEY_DELETE);
  C(KEY_PAGEUP); C(KEY_PAGEDOWN); C(KEY_INSERT);
  C(KEY_LEFTSHIFT); C(KEY_RIGHTSHIFT); C(KEY_LEFTCTRL); C(KEY_RIGHTCTRL); C(KEY_LEFTALT); C(KEY_RIGHTALT);
  C(KEY_LEFTMETA); C(KEY_RIGHTMETA); C(KEY_CAPSLOCK);
  C(KEY_F1); C(KEY_F10); C(KEY_F11); C(KEY_F12);
  C(KEY_GRAVE); C(KEY_1); C(KEY_0); C(KEY_MINUS); C(KEY_EQUAL); C(KEY_Q); C(KEY_P); C(KEY_LEFTBRACE); C(KEY_RIGHTBRACE);
  C(KEY_A); C(KEY_L); C(KEY_SEMICOLON); C(KEY_APOSTROPHE); C(KEY_BACKSLASH); C(KEY_102ND);
  C(KEY_Z); C(KEY_M); C(KEY_COMMA); C(KEY_DOT); C(KEY_SLASH);
  C(KEY_KP0); C(KEY_KP1); C(KEY_KP2); C(KEY_KP3); C(KEY_KP4); C(KEY_KP5); C(KEY_KP6); C(KEY_KP7); C(KEY_KP8); C(KEY_KP9);
  C(KEY_KPDOT); C(KEY_KPPLUS); C(KEY_KPMINUS); C(KEY_KPASTERISK); C(KEY_KPSLASH);
  return 0;
}
