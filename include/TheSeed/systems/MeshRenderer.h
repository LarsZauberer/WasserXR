#ifndef TS_MeshRenderer_H
#define TS_MeshRenderer_H

#include "TheSeed/ecs/Scene.h"

int ts_select_ts_mesh_renderer(const TS_Scene *scene, const TS_Entity entity);
void ts_system_ts_mesh_renderer(TS_Scene *scene, TS_Entity **entities, const size_t *groups);

#endif
