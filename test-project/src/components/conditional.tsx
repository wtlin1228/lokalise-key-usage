import { translate } from "../utils/i18n";

const LABELS = translate({
  bird: "lokalise.key.bird",
  cat: "lokalise.key.cat",
  dog: "lokalise.key.dog",

  getBird: ["lokalise.key.get-bird", "lazy"],
  getCat: ["lokalise.key.get-cat", "lazy"],
  getDog: ["lokalise.key.get-dog", "lazy"],
});

const Foo = (cond: boolean) => {
  return <div>{cond ? LABELS.bird : LABELS.cat}</div>;
};

const Bar = (cond: boolean) => {
  return <div>{cond ? LABELS.getBird({}) : LABELS.getCat({})}</div>;
};
