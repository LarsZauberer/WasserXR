#ifndef WXR_CONSOLE_SYSTEM
#define WXR_CONSOLE_SYSTEM

#include "WasserXR/ecs/Scene.h"
#include <stddef.h>

void wxr_attach_wxr_console_system(WXR_Scene *scene);
void wxr_detach_wxr_console_system(WXR_Scene *scene);
void wxr_system_wxr_console_system(WXR_Scene *scene, WXR_Entity **entities,
                                 const size_t *groups);
WXR_System_Groups wxr_select_wxr_console_system(const WXR_Scene *scene,
                                             WXR_Entity entity);

#endif
