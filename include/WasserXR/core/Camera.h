#ifndef WXR_CAMERA_H
#define WXR_CAMERA_H

#include "WasserXR/ecs/Scene.h"

typedef struct WXR_Camera WXR_Camera;

void *wxr_create_WXR_Camera();
void wxr_destroy_WXR_Camera(void *cam);
void wxr_schema_WXR_Camera(WXR_Component_Schema *schema);

#endif
