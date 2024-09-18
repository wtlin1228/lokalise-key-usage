const LABEL_KEYS = {
  bird: "lokalise.key.bird",
  c: {
    cat: "lokalise.key.cat",
  },
  dog: "lokalise.key.dog",
};

const Foo = () => <TransBlock i18nKey={LABEL_KEYS.bird} />;

const Bar = () => <TransBlock i18nKey={LABEL_KEYS.c.cat} />;

const TransBlock = ({ i18nKey }: { i18nKey: string }) => {
  return <div>{i18nKey}</div>;
};
