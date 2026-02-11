#ifndef TS_MeshRenderer_H
#define TS_MeshRenderer_H

#include "TheSeed/ecs/Scene.h"

int ts_select_ts_mesh_renderer(TS_Scene_t *scene, const size_t entity);
void ts_system_ts_mesh_renderer(TS_Scene_t *scene, size_t *entities, size_t n);

#endif
