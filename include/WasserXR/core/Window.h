#ifndef WXR_WINDOW_H
#define WXR_WINDOW_H

#include "WasserXR/ecs/Scene.h"
#include <GLFW/glfw3.h>

typedef struct WXR_Window WXR_Window;

void *wxr_create_WXR_Window();
void wxr_destroy_WXR_Window(void *window);
void wxr_schema_WXR_Window(WXR_Component_Schema *schema);

#endif
