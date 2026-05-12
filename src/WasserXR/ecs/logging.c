/*
 * Copyright (c) 2026 Ian Wasser
 * Licensed under the WasserXR License.
 * You may not use this file except in compliance with the License.
 * See LICENSE.md for details.
 */

#include "WasserXR/ecs/logging.h"
#include <glib.h>
#include <stdio.h>
#include <string.h>

// Globals

static WXR_Log_Level wxr_lowest_level = WXR_LOG_INFO;
static GArray *wxr_loggers = NULL;

static WXR_Log_Entry wxr_create_entry(WXR_Log_Level log_level, const char *fmt,
                                      va_list args) {
  char *formatted_string = g_strdup_vprintf(fmt, args);
  const WXR_Log_Entry entry = {log_level, formatted_string};
  return entry;
}

static void wxr_destroy_entry(WXR_Log_Entry entry) { free(entry.msg); }

static void wxr_send_entry_to_loggers(const WXR_Log_Entry entry) {
  g_assert(wxr_loggers);
  for (unsigned int i = 0; i < wxr_loggers->len; i++) {
    const WXR_Logger logger = g_array_index(wxr_loggers, WXR_Logger, i);
    logger(entry);
  }
}

void wxr_logging_init(const WXR_Log_Level level) {
  wxr_lowest_level = level;
  wxr_loggers = g_array_new(FALSE, FALSE, sizeof(WXR_Logger));
}

void wxr_add_logger(const WXR_Logger logger) {
  g_array_append_val(wxr_loggers, logger);
}

void wxr_stdout_logger(const WXR_Log_Entry entry) {
  char *level = NULL;
  char *color = NULL;
  switch (entry.level) {
  case WXR_LOG_DEBUG:
    level = "DEBUG";
    color = "37";
    break;
  case WXR_LOG_INFO:
    level = "INFO";
    color = "34";
    break;
  case WXR_LOG_WARN:
    level = "WARN";
    color = "33";
    break;
  case WXR_LOG_ERROR:
    level = "ERROR";
    color = "31";
    break;
  case WXR_LOG_CRITICAL:
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

void wxr_debug(const char *fmt, ...) {
  if (!wxr_loggers) {
    return;
  }
  if (wxr_lowest_level > WXR_LOG_DEBUG) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  WXR_Log_Entry entry = wxr_create_entry(WXR_LOG_DEBUG, fmt, args);
  va_end(args);
  wxr_send_entry_to_loggers(entry);
  wxr_destroy_entry(entry);
}

void wxr_info(const char *fmt, ...) {
  if (!wxr_loggers) {
    return;
  }
  if (wxr_lowest_level > WXR_LOG_INFO) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  WXR_Log_Entry entry = wxr_create_entry(WXR_LOG_INFO, fmt, args);
  va_end(args);
  wxr_send_entry_to_loggers(entry);
  wxr_destroy_entry(entry);
}

void wxr_warn(const char *fmt, ...) {
  if (!wxr_loggers) {
    return;
  }
  if (wxr_lowest_level > WXR_LOG_WARN) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  WXR_Log_Entry entry = wxr_create_entry(WXR_LOG_WARN, fmt, args);
  va_end(args);
  wxr_send_entry_to_loggers(entry);
  wxr_destroy_entry(entry);
}

void wxr_error(const char *fmt, ...) {
  if (!wxr_loggers) {
    return;
  }
  if (wxr_lowest_level > WXR_LOG_ERROR) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  WXR_Log_Entry entry = wxr_create_entry(WXR_LOG_ERROR, fmt, args);
  va_end(args);
  wxr_send_entry_to_loggers(entry);
  wxr_destroy_entry(entry);
}

void wxr_critical(const char *fmt, ...) {
  if (!wxr_loggers) {
    return;
  }
  if (wxr_lowest_level > WXR_LOG_CRITICAL) {
    return;
  }
  va_list args;
  va_start(args, fmt);
  WXR_Log_Entry entry = wxr_create_entry(WXR_LOG_CRITICAL, fmt, args);
  va_end(args);
  wxr_send_entry_to_loggers(entry);
  wxr_destroy_entry(entry);

  // The critical logging exits immediately
  exit(1);
}
