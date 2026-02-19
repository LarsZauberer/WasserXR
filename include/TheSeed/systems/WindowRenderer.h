#include "TheSeed/ecs/Scene.h"
#include <stddef.h>

TS_System_Groups ts_select_ts_window_post_renderer(TS_Scene *, const size_t);
void ts_system_ts_window_post_renderer(TS_Scene *, size_t **, size_t *);

TS_System_Groups ts_select_ts_window_pre_renderer(TS_Scene *, const size_t);
void ts_system_ts_window_pre_renderer(TS_Scene *, size_t **, size_t *);

TS_System_Groups ts_select_ts_window_quiter(TS_Scene *, const size_t);
void ts_system_ts_window_quiter(TS_Scene *, size_t **, size_t *);

TS_System_Groups ts_select_ts_window_reloader(TS_Scene *, const size_t);
void ts_system_ts_window_reloader(TS_Scene *, size_t **, size_t *);
