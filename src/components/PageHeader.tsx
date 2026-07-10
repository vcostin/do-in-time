import { Link } from 'react-router-dom';

interface PageHeaderProps {
  title: string;
  description?: string;
  backTo?: string;
  backLabel?: string;
}

export function PageHeader({
  title,
  description,
  backTo = '/',
  backLabel = 'Back',
}: PageHeaderProps) {
  return (
    <div className="mb-6">
      <Link
        to={backTo}
        className="inline-flex items-center gap-1 text-sm text-blue-600 dark:text-blue-400 hover:underline mb-3"
      >
        ← {backLabel}
      </Link>
      <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{title}</h2>
      {description && (
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">{description}</p>
      )}
    </div>
  );
}
