export type UpdateManifest = {
	update_id: UpdateId
	is_deployable: boolean
	compatibility: Compatibility[]
	created_date_time: string
	manifest_version: string
}

export type UpdateId = {
	provider: string
	name: string
	version: string
}

export type Compatibility = {
	manufacturer: string
	model: string
	compatibilityid: string
}
