#ifndef WXR_MeshRenderer_H
#define WXR_MeshRenderer_H

#include "WasserXR/ecs/Scene.h"

int wxr_select_wxr_mesh_renderer(const WXR_Scene *scene, const WXR_Entity entity);
void wxr_system_wxr_mesh_renderer(WXR_Scene *scene, WXR_Entity **entities, const size_t *groups);

#endif
