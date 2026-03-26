// Datastructures
typedef enum {
  TS_LOG_DEBUG,
  TS_LOG_INFO,
  TS_LOG_WARN,
  TS_LOG_ERROR,
  TS_LOG_CRITICAL,
} TS_Log_Level;

typedef struct {
  TS_Log_Level level;
  char *msg;
} TS_Log_Entry;

// Logging functions

void ts_debug(const char * /*fmt*/, ...);
void ts_info(const char * /*fmt*/, ...);
void ts_warn(const char * /*fmt*/, ...);
void ts_error(const char * /*fmt*/, ...);
void ts_critical(const char * /*fmt*/, ...);

#ifndef TS_NO_ASSERTS

#define ts_assert(exp, fmt, ...)                                               \
  if (!(exp)) {                                                                \
    ts_critical(fmt __VA_OPT__(, ) __VA_ARGS__);                               \
  }

#define ts_assert_recoverable(exp, fmt, ...)                                   \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
  }

#define ts_assert_abort(exp, fmt, ...)                                         \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
    return;                                                                    \
  }

#define ts_assert_abort_value(exp, value, fmt, ...)                            \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
    return value;                                                              \
  }

#else

#define ts_assert(exp, fmt, ...)

#define ts_assert_recoverable(exp, fmt, ...)

#define ts_assert_abort(exp, fmt, ...)

#define ts_assert_abort_value(exp, value, fmt, ...)

#endif

#define ts_assert_test(exp, should, should_val, out, out_val, fmt, ...)        \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
    ts_error(should, should_val);                                              \
    ts_error(out, out_val);                                                    \
    exit(1);                                                                   \
  }

// Logger registry

typedef void (*TS_Logger)(const TS_Log_Entry);

void ts_add_logger(TS_Logger /*logger*/);

// Default loggers

void ts_stdout_logger(TS_Log_Entry entry);

// Init
void ts_logging_init(TS_Log_Level lowest_level);
