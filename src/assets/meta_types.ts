export type TableMeta = {
  id: string;
  name: string;
  fields: FieldMeta[];
  comment?: string;
};

export type FieldMeta = {
  name: string;
  comment?: string;
  type: FieldType;
};

export type FieldType =
  | {
      name: "any" | "null" | "boolean" | "string" | "number" | "date";
    }
  | {
      name: "option";
      inner: FieldType;
    }
  | {
      name: "object";
      fields: null | FieldMeta[];
    }
  | {
      name: "record";
      table: string;
    }
  | {
      name: "array";
      item: FieldType;
    }
  | {
      name: "union";
      variants: FieldType[];
    }
  | {
      name: "union";
      variants: FieldType[];
      kind: "string";
    }
  | {
      name: "union";
      variants: FieldType[];
      kind: "number";
    }
  | {
      name: "literal";
      kind: "number";
      value: number;
    }
  | {
      name: "literal";
      kind: "string";
      value: string;
    }
  | {
      name: "literal";
      kind: "array";
      items: FieldType[];
    };
