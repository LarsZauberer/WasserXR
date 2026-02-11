#include "TheSeed/ecs/Scene.h"
#include <stddef.h>

int ts_select_ts_window_post_renderer(TS_Scene_t *, const size_t);
void ts_system_ts_window_post_renderer(TS_Scene_t *, size_t *, size_t);

int ts_select_ts_window_pre_renderer(TS_Scene_t *, const size_t);
void ts_system_ts_window_pre_renderer(TS_Scene_t *, size_t *, size_t);

int ts_select_ts_window_quiter(TS_Scene_t *, const size_t);
void ts_system_ts_window_quiter(TS_Scene_t *, size_t *, size_t);

int ts_select_ts_window_reloader(TS_Scene_t *, const size_t);
void ts_system_ts_window_reloader(TS_Scene_t *, size_t *, size_t);
