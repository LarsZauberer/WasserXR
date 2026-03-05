#ifndef TS_CONSOLE_SYSTEM
#define TS_CONSOLE_SYSTEM

#include "TheSeed/ecs/Scene.h"
#include <stddef.h>

void ts_attach_ts_console_system(TS_Scene *scene);
void ts_detach_ts_console_system(TS_Scene *scene);
void ts_system_ts_console_system(TS_Scene *scene, TS_Entity **entities,
                                 const size_t *groups);

#endif
