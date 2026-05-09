#ifndef WXR_CONSOLE_H
#define WXR_CONSOLE_H

#include "WasserXR/ecs/Scene.h"

typedef struct WXR_Console WXR_Console;

void *wxr_create_WXR_Console();
void wxr_destroy_WXR_Console(void *ptr);
void wxr_schema_WXR_Console(WXR_Component_Schema *schema);

#endif
