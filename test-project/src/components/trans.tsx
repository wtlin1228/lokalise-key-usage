const LABEL_KEYS = {
  bird: "lokalise.key.bird",
  c: {
    cat: "lokalise.key.cat",
  },
  dog: "lokalise.key.dog",
};

const Foo = () => <Trans i18nKey={LABEL_KEYS.bird} />;

const Bar = () => <Trans i18nKey={LABEL_KEYS.c.cat} />;

const Trans = ({ i18nKey }: { i18nKey: string }) => {
  return <div>{i18nKey}</div>;
};
