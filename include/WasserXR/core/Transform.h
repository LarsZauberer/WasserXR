#ifndef WXR_Transform_H
#define WXR_Transform_H

#include "WasserXR/ecs/Scene.h"
typedef struct WXR_Transform WXR_Transform;

void *wxr_create_WXR_Transform();
void wxr_destroy_WXR_Transform(void *ptr);
void wxr_schema_WXR_Transform(WXR_Component_Schema *schema);

#endif
