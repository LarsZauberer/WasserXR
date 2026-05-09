#include "WasserXR/ecs/Scene.h"
#include <stddef.h>

WXR_System_Groups wxr_select_wxr_window_post_renderer(const WXR_Scene *scene,
                                                   WXR_Entity entity);
void wxr_system_wxr_window_post_renderer(WXR_Scene *scene, WXR_Entity **entities,
                                       const size_t *groups);

WXR_System_Groups wxr_select_wxr_window_pre_renderer(const WXR_Scene *scene,
                                                  WXR_Entity entity);
void wxr_system_wxr_window_pre_renderer(WXR_Scene *scene, WXR_Entity **entities,
                                      const size_t *groups);

WXR_System_Groups wxr_select_wxr_window_quiter(const WXR_Scene *scene,
                                            WXR_Entity entity);
void wxr_system_wxr_window_quiter(WXR_Scene *scene, WXR_Entity **entities,
                                const size_t *groups);

WXR_System_Groups wxr_select_wxr_window_reloader(const WXR_Scene *scene,
                                              WXR_Entity entity);
void wxr_system_wxr_window_reloader(WXR_Scene *scene, WXR_Entity **entities,
                                  const size_t *groups);
