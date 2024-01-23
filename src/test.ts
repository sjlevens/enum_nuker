export const MultitaxTypes = {
  GST: "gst",
  VAT: "vatNumber",
} as const;

export type MultitaxTypes = (typeof MultitaxTypes)[keyof typeof MultitaxTypes];