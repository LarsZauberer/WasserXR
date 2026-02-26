#include "TheSeed/ecs/Scene.h"
#include <stddef.h>

TS_System_Groups ts_select_ts_window_post_renderer(TS_Scene *scene,
                                                   TS_Entity entity);
void ts_system_ts_window_post_renderer(TS_Scene *scene, size_t **entities,
                                       const size_t *size);

TS_System_Groups ts_select_ts_window_pre_renderer(TS_Scene *scene,
                                                  TS_Entity entity);
void ts_system_ts_window_pre_renderer(TS_Scene *scene, size_t **entities,
                                      const size_t *size);

TS_System_Groups ts_select_ts_window_quiter(TS_Scene *scene, TS_Entity entity);
void ts_system_ts_window_quiter(TS_Scene *scene, size_t **entities,
                                const size_t *size);

TS_System_Groups ts_select_ts_window_reloader(TS_Scene *scene,
                                              TS_Entity entity);
void ts_system_ts_window_reloader(TS_Scene *scene, size_t **entities,
                                  const size_t *size);
