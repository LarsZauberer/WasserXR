/*
 * Copyright (c) 2026 Ian Wasser
 * Licensed under the WasserXR License.
 * You may not use this file except in compliance with the License.
 * See LICENSE.md for details.
 */

// Datastructures
typedef enum {
  WXR_LOG_DEBUG,
  WXR_LOG_INFO,
  WXR_LOG_WARN,
  WXR_LOG_ERROR,
  WXR_LOG_CRITICAL,
} WXR_Log_Level;

typedef struct {
  WXR_Log_Level level;
  char *msg;
} WXR_Log_Entry;

// Logging functions

void wxr_debug(const char * /*fmt*/, ...);
void wxr_info(const char * /*fmt*/, ...);
void wxr_warn(const char * /*fmt*/, ...);
void wxr_error(const char * /*fmt*/, ...);
void wxr_critical(const char * /*fmt*/, ...);

#ifndef WXR_NO_ASSERTS

#define wxr_assert(exp, fmt, ...)                                              \
  if (!(exp)) {                                                                \
    wxr_critical(fmt __VA_OPT__(, ) __VA_ARGS__);                              \
  }

#define wxr_assert_recoverable(exp, fmt, ...)                                  \
  if (!(exp)) {                                                                \
    wxr_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                 \
  }

#define wxr_assert_abort(exp, fmt, ...)                                        \
  if (!(exp)) {                                                                \
    wxr_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                 \
    return;                                                                    \
  }

#define wxr_assert_abort_value(exp, value, fmt, ...)                           \
  if (!(exp)) {                                                                \
    wxr_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                 \
    return value;                                                              \
  }

#else

#define wxr_assert(exp, fmt, ...)

#define wxr_assert_recoverable(exp, fmt, ...)

#define wxr_assert_abort(exp, fmt, ...)

#define wxr_assert_abort_value(exp, value, fmt, ...)

#endif

#define wxr_assert_test(exp, should, should_val, out, out_val, fmt, ...)       \
  if (!(exp)) {                                                                \
    wxr_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                 \
    wxr_error(should, should_val);                                             \
    wxr_error(out, out_val);                                                   \
    exit(1);                                                                   \
  }

// Logger registry

typedef void (*WXR_Logger)(const WXR_Log_Entry);

void wxr_add_logger(WXR_Logger /*logger*/);

// Default loggers

void wxr_stdout_logger(WXR_Log_Entry entry);

// Init
void wxr_logging_init(WXR_Log_Level lowest_level);
