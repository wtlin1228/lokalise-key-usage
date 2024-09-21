import { translate } from "@foo/i18n";

const LABELS = translate({
  bird: "i18n.bird",
  cat: ["i18n.cat", "lazy"],
  dog: ["i18n.dog", "lazy"],
});
