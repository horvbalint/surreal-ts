export type TableMeta = {
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
      name: "literal";
      type: "number";
      value: number;
    }
  | {
      name: "literal";
      type: "string";
      value: string;
    }
  | {
      name: "literal";
      type: "array";
      inner: FieldType[];
    };
