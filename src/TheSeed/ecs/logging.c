#include "TheSeed/ecs/logging.h"
#include <glib.h>
#include <stdio.h>
#include <string.h>

// Globals

static TS_Log_Level ts_lowest_level = TS_LOG_INFO;
static GArray *ts_loggers = NULL;

static TS_Log_Entry ts_create_entry(TS_Log_Level log_level, const char *fmt,
                                    va_list args) {
  char *formatted_string = g_strdup_vprintf(fmt, args);
  const TS_Log_Entry entry = {log_level, formatted_string};
  return entry;
}

static void ts_destroy_entry(TS_Log_Entry entry) { free(entry.msg); }

static void ts_send_entry_to_loggers(const TS_Log_Entry entry) {
  g_assert(ts_loggers);
  for (unsigned int i = 0; i < ts_loggers->len; i++) {
    const TS_Logger logger = g_array_index(ts_loggers, TS_Logger, i);
    logger(entry);
  }
}

void ts_logging_init(const TS_Log_Level level) {
  ts_lowest_level = level;
  ts_loggers = g_array_new(FALSE, FALSE, sizeof(TS_Logger));
}

void ts_add_logger(const TS_Logger logger) {
  g_array_append_val(ts_loggers, logger);
}

void ts_stdout_logger(const TS_Log_Entry entry) {
  char *level = NULL;
  char *color = NULL;
  switch (entry.level) {
  case TS_LOG_DEBUG:
    level = "DEBUG";
    color = "37";
    break;
  case TS_LOG_INFO:
    level = "INFO";
    color = "34";
    break;
  case TS_LOG_WARN:
    level = "WARN";
    color = "33";
    break;
  case TS_LOG_ERROR:
    level = "ERROR";
    color = "31";
    break;
  case TS_LOG_CRITICAL:
    level = "CRITICAL";
    color = "41";
    break;
  default:
    level = "CUSTOM";
    color = "36";
    break;
  }
  printf("[\033[%sm%s\033[0m]: %s\n", color, level, entry.msg);
}

void ts_debug(const char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  if (ts_lowest_level > TS_LOG_DEBUG) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_DEBUG, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
}

void ts_info(const char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  if (ts_lowest_level > TS_LOG_INFO) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_INFO, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
}

void ts_warn(const char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  if (ts_lowest_level > TS_LOG_WARN) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_WARN, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
}

void ts_error(const char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  if (ts_lowest_level > TS_LOG_ERROR) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_ERROR, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
}

void ts_critical(const char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  if (ts_lowest_level > TS_LOG_CRITICAL) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_CRITICAL, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);

  // The critical logging exits immediately
  exit(1);
}
