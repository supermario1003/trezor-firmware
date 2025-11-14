#include <trezor_rtl.h>

#include "py/mpthread.h"
#include "py/runtime.h"
#include "py/stackctrl.h"
#ifdef USE_SECP256K1_ZKP
#include "zkp_context.h"
#endif

MP_NOINLINE int main_(int argc, char **argv);

#include "secbool.h"

ts_t  __wur func() {
  return TS_OK;
}

void test() {

  ts_t status = func();

  if (ts_eq(status, TS_OK)) {
    // do nothing
  } else if (ts_eq(status, TS_ERROR)) {
    // do nothing
  }

  if (ts_error(func())) {
    // do nothing
  }

  func();
}


int main(int argc, char **argv) {

  test();


#ifdef USE_SECP256K1_ZKP
  ensure(sectrue * (zkp_context_init() == 0), NULL);
#endif

#if MICROPY_PY_THREAD
  mp_thread_init();
#endif
  // We should capture stack top ASAP after start, and it should be
  // captured guaranteedly before any other stack variables are allocated.
  // For this, actual main (renamed main_) should not be inlined into
  // this function. main_() itself may have other functions inlined (with
  // their own stack variables), that's why we need this main/main_ split.
  mp_stack_ctrl_init();
  return main_(argc, argv);
}
