#include "TheSeed/core/logging.h"
#include <glib.h>
#include <stdio.h>
#include <string.h>

// Globals

GArray *ts_loggers = NULL;

static TS_Log_Entry ts_create_entry(TS_Log_Level log_level, char *fmt,
                                    va_list args) {
  char *formatted_string = g_strdup_vprintf(fmt, args);
  TS_Log_Entry entry = {log_level, formatted_string};
  return entry;
}

static void ts_destroy_entry(TS_Log_Entry entry) {
  free(entry.msg);
  return;
}

static void ts_send_entry_to_loggers(TS_Log_Entry entry) {
  g_assert(ts_loggers);
  for (unsigned int i = 0; i < ts_loggers->len; i++) {
    TS_Logger logger = g_array_index(ts_loggers, TS_Logger, i);
    logger(entry);
  }
  return;
}

void ts_logging_init() {
  ts_loggers = g_array_new(FALSE, FALSE, sizeof(TS_Logger));
  return;
}

void ts_add_logger(TS_Logger logger) {
  g_array_append_val(ts_loggers, logger);
  return;
}

void ts_stdout_logger(TS_Log_Entry entry) {
  char *level = NULL;
  switch (entry.level) {
  case TS_LOG_DEBUG:
    level = "DEBUG";
    break;
  case TS_LOG_INFO:
    level = "INFO";
    break;
  case TS_LOG_WARN:
    level = "WARN";
    break;
  case TS_LOG_ERROR:
    level = "ERROR";
    break;
  case TS_LOG_CRITICAL:
    level = "CRITICAL";
    break;
  default:
    level = "CUSTOM";
    break;
  }
  printf("[%s]: %s\n", level, entry.msg);
  return;
}

void ts_debug(char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_DEBUG, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
  return;
}

void ts_info(char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_INFO, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
  return;
}

void ts_warn(char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_WARN, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
  return;
}

void ts_error(char *fmt, ...) {
  if (!ts_loggers) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  TS_Log_Entry entry = ts_create_entry(TS_LOG_ERROR, fmt, args);
  va_end(args);
  ts_send_entry_to_loggers(entry);
  ts_destroy_entry(entry);
  return;
}

void ts_critical(char *fmt, ...) {
  if (!ts_loggers) {
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
