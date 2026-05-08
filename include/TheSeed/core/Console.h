#ifndef TS_CONSOLE_H
#define TS_CONSOLE_H

#include "TheSeed/ecs/Scene.h"

typedef struct TS_Console TS_Console;

void *ts_create_TS_Console();
void ts_destroy_TS_Console(void *ptr);
void ts_schema_TS_Console(TS_Component_Schema *schema);

#endif
