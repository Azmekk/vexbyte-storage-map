type TopBarTitleProps = {
  title: string;
};

export function TopBarTitle({ title }: TopBarTitleProps) {
  return <span className="truncate font-medium text-sm">{title}</span>;
}
