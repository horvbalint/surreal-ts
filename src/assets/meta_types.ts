export type TableMeta = {
  name: string
  fields: FieldMeta[]
  comment?: string
}

export type FieldMeta = {
  name: string
  isOptional: boolean
  isArray: boolean
  type: string
  comment?: string
  isRecord?: true
  fields?: FieldMeta[]
}
