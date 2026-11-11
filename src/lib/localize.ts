const AR_DIGITS = '٠١٢٣٤٥٦٧٨٩';

export function arDigits(value: string | number): string {
  return String(value).replace(/[0-9]/g, (digit) => AR_DIGITS[Number(digit)]);
}

export function arNumeric(value: string): string {
  return arDigits(value.replace(/(\d)\.(\d)/g, '$1٫$2'));
}

const fragments: [string, string][] = [
  // Simplex
  ['The optimal value is ', 'القيمة المثلى هي '],
  [' at ', ' عند '],
  ['The tableau was solved as a maximisation of ', 'حُلَّ الجدول باعتباره تعظيمًا لـ '],
  [', and the reported minimum is ', '، والقيمة الصغرى المبلَّغ عنها هي '],
  [
    'The problem is infeasible: no point satisfies all constraints.',
    'المسألة غير ممكنة: لا توجد نقطة تحقق جميع القيود.'
  ],
  [
    'The objective is unbounded: it can be made arbitrarily large (or small) while satisfying the constraints.',
    'الهدف غير محدود: يمكن جعله كبيرًا (أو صغيرًا) اعتباطيًا مع تحقيق القيود.'
  ],
  ['The most negative entry in the objective row is ', 'أكثر عنصر سالب في صف الهدف هو '],
  [', in the ', '، في '],
  [' column, so ', '، لذا يدخل '],
  [' enters the basis.', ' إلى الأساس.'],
  ['Ratio test: ', 'اختبار النسبة: '],
  ['. The smallest ratio is ', '. أصغر نسبة هي '],
  [' in row ', ' في الصف '],
  [', so ', '، لذا '],
  [' leaves the basis.', ' يخرج من الأساس.'],
  ['Pivot on the ', 'المحور على '],
  [', column ', '، العمود '],
  [
    'An artificial variable is still positive, so the constraints cannot all be satisfied. The problem is infeasible.',
    'لا يزال هناك متغير اصطناعي موجب، لذا لا يمكن تحقيق جميع القيود. المسألة غير ممكنة.'
  ],
  [
    'No negative entries remain in the objective row, so this tableau is optimal.',
    'لم يبق أي عنصر سالب في صف الهدف، لذا هذا الجدول مثلى.'
  ],
  [
    'The entering variable ',
    'المتغير الداخل '
  ],
  [
    ' has no positive coefficient in any constraint row, so it can increase without limit. The objective is unbounded.',
    ' ليس له معامل موجب في أي صف قيد، لذا يمكنه الزيادة بلا حد. الهدف غير محدود.'
  ],

  // Matrix
  ['Start with the augmented matrix ', 'ابدأ بالمصفوفة المعزَّزة '],
  ['Start with ', 'ابدأ بـ '],
  ['. Work left to right: choose a nonzero pivot in each column, make it ', '. اعمل من اليسار إلى اليمين: اختر محورًا غير صفري في كل عمود، واجعله '],
  [', and clear the rest of that column.', '، ثم صفِّر بقية ذلك العمود.'],
  ['Subtract ', 'اطرح '],
  [' times ', ' مضروبًا في '],
  [' from ', ' من '],
  [' (', ' ('],
  ['). This clears the entry in column ', '). هذا يصفّر العنصر في العمود '],
  [
    '). This clears the entry below the pivot; row replacement leaves the determinant unchanged.',
    '). هذا يصفّر العنصر تحت المحور؛ استبدال الصف لا يغيّر المحدد.'
  ],
  ['Add ', 'اجمع '],
  [
    '. Clear below each pivot in turn; row swaps flip the sign, and the determinant is the product of the pivots.',
    '. صفِّر ما تحت كل محور بالترتيب؛ تبديل الصفوف يقلب الإشارة، والمحدد هو حاصل ضرب المحاور.'
  ],
  ['The pivot is ', 'المحور هو '],
  ['. Multiply the running determinant by it.', '. اضرب المحدد الجاري فيه.'],
  [
    'Bring a nonzero entry into the pivot position: swap ',
    'اجلب عنصرًا غير صفري إلى موضع المحور: بدّل '
  ],
  ['. A swap changes the sign of the determinant.', '. التبديل يغيّر إشارة المحدد.'],
  [
    '. Row operations on the left block also change the right block; when the left block becomes ',
    '. عمليات الصفوف على الكتلة اليسرى تغيّر الكتلة اليمنى أيضًا؛ وعندما تصبح الكتلة اليسرى '
  ],
  [', the right block is ', '، تصبح الكتلة اليمنى '],
  ['Divide row ', 'اقسم الصف '],
  [' by ', ' على '],
  [' so the pivot becomes ', ' حتى يصبح المحور '],
  [
    '. Row-reduce it; each free column gives one nullspace basis vector.',
    '. اختزلها بالصفوف؛ كل عمود حر يعطي متجه أساس واحدًا للفضاء الصفري.'
  ],
  [
    '. Row-reduce the whole augmented matrix; the reduced system has the same solutions as the original.',
    '. اختزل المصفوفة المعزَّزة كاملة بالصفوف؛ النظام المختزل له الحلول نفسها.'
  ],
  ['Row-reducing ', 'اختزال '],
  [' turned the left block into ', ' حوّل الكتلة اليسرى إلى '],
  ['; the right block is ', '؛ والكتلة اليمنى هي '],
  ['Every column is a pivot column, so the solution is unique.', 'كل عمود هو عمود محور، لذا الحل وحيد.'],
  [
    'Every column is a pivot column, so the nullspace is the zero subspace.',
    'كل عمود هو عمود محور، لذا الفضاء الصفري هو الفضاء الجزئي الصفري.'
  ],
  ['The values of the variables are shown above.', 'قيم المتغيرات معروضة أعلاه.'],
  ['The left block could not become ', 'تعذّر أن تصبح الكتلة اليسرى '],
  [' is singular and has no inverse.', ' شاذة وليس لها معكوس.'],
  [' is singular.', ' شاذة.'],
  ['No nonzero pivot in column ', 'لا يوجد محور غير صفري في العمود '],
  [', so the system is inconsistent.', '، لذا النظام غير متسق.'],
  ['The equation ', 'المعادلة '],
  [' cannot be satisfied.', ' لا يمكن تحقيقها.'],
  [
    'A particular solution and a nullspace basis are shown above.',
    'يظهر أعلاه حل خاص وأساس للفضاء الصفري.'
  ],
  [
    'Choose a parameter for each free column and add the corresponding nullspace vector to ',
    'اختر وسيطًا لكل عمود حر وأضف متجه الفضاء الصفري المقابل إلى '
  ],
  ['; each one gives one basis vector.', '؛ كل واحد يعطي متجه أساس واحدًا.'],
  ['No inverse exists.', 'لا يوجد معكوس.'],

  // Complex
  ['Add the real parts: ', 'اجمع الأجزاء الحقيقية: '],
  ['Subtract the real parts: ', 'اطرح الأجزاء الحقيقية: '],
  ['Add the imaginary parts: ', 'اجمع الأجزاء التخيلية: '],
  ['Subtract the imaginary parts: ', 'اطرح الأجزاء التخيلية: '],
  ['Multiply numerator and denominator by the conjugate ', 'اضرب البسط والمقام في مرافق '],
  ['Multiply out ', 'اضرب '],
  ['The denominator becomes ', 'يصبح المقام '],
  ['The modulus is ', 'المعيار هو '],
  [' and the argument is ', ' والسعة هي '],
  ['The argument is ', 'السعة هي '],
  ['Write ', 'اكتب '],
  [' in polar form: ', ' بالصورة القطبية: '],
  ['De Moivre: ', 'دي موافر: '],
  ['. Here ', '. هنا '],
  ['The roots are ', 'الجذور هي '],
  [', with ', '، حيث '],
  ['Let ', 'لتكن '],
  ['. The roots are ', '. الجذور هي '],
  ['A principal ', 'الجذر الرئيسي رقم '],
  [' root is ', ' هو '],
  ['The only ', 'الجذر الوحيد '],
  ['th root of ', ' للعدد '],
  [' is ', ' هو '],
  [' and ', ' و ']
];

const orderedFragments = [...fragments].sort((a, b) => b[0].length - a[0].length);

export function translateFragment(fragment: string): string {
  let value = fragment;
  value = value.replace(
    /A principal (\d+)(?:st|nd|rd|th) root is /g,
    (_match, n: string) => `الجذر الرئيسي رقم ${arDigits(n)} هو `
  );
  value = value.replace(
    /The only (\d+)(?:st|nd|rd|th) root of /g,
    (_match, n: string) => `الجذر الوحيد من الدرجة ${arDigits(n)} للعدد `
  );
  for (const [from, to] of orderedFragments) {
    if (value.includes(from)) value = value.split(from).join(to);
  }
  value = value.replace(/Row (\d+) reduces to /g, (_match, row: string) => `يختزل الصف ${arDigits(row)} إلى `);
  value = value.replace(
    /; the solution has (\d+) parameters?\./g,
    (_match, count: string) => `؛ الحل له ${arDigits(count)} ${count === '1' ? 'وسيط' : 'وسائط'}.`
  );
  return arNumeric(value);
}

/** Translate prose that may contain inline `$...$` maths. */
export function localizeProse(text: string): string {
  return text
    .split(/(\$[^$]*\$)/g)
    .map((part, index) => (index % 2 === 1 ? part : translateFragment(part)))
    .join('');
}

const latexWords: [RegExp, string][] = [
  [/Maximise/g, 'عظّم'],
  [/Minimise/g, 'صغّر'],
  [/\\text\{rad\}/g, '\\text{راديان}'],
  [/\\text\{Pivot columns: \}/g, '\\text{أعمدة المحور: }'],
  [/\\text\{Free columns: \}/g, '\\text{الأعمدة الحرة: }'],
  [/\\text\{free columns: \}/g, '\\text{الأعمدة الحرة: }'],
  [/\\cos/g, '\\operatorname{جتا}'],
  [/\\sin/g, '\\operatorname{جا}'],
  [/\\arg/g, '\\operatorname{سعة}'],
  [/\\det/g, '\\operatorname{محدد}'],
  [/\\dim/g, '\\operatorname{بُعد}'],
  [/\\text\{b\.v\.\}/g, '\\text{المتغير الأساسي}'],
  [/\\text\{RHS\}/g, '\\text{الطرف الأيمن}'],
  [/R_\{/g, '\\mathrm{ص}_{'],
  [/\\operatorname\{rank\}/g, '\\operatorname{رتبة}'],
  [/\\operatorname\{null\}/g, '\\operatorname{فراغ}'],
  [/\\operatorname\{nullity\}/g, '\\operatorname{عُطالة}']
];

export function localizeLatex(source: string): string {
  let value = source;
  for (const [pattern, replacement] of latexWords) {
    value = value.replace(pattern, replacement);
  }
  return value;
}

const errors: [RegExp, string][] = [
  [/^division by zero$/, 'القسمة على صفر'],
  [/^use a power between 0 and 12$/, 'استخدم أسًا بين ٠ و١٢'],
  [/^use a root index between 1 and 12$/, 'استخدم دليل جذر بين ١ و١٢'],
  [/^use between 1 and 10 constraints$/, 'استخدم بين ١ و١٠ قيود'],
  [/^use between 1 and 6 (rows|columns|variables)$/, 'استخدم بين ١ و٦'],
  [/^add at least one constraint$/, 'أضف قيدًا واحدًا على الأقل'],
  [/^add at least one variable$/, 'أضف متغيرًا واحدًا على الأقل'],
  [/^the determinant needs a square matrix$/, 'يحتاج المحدد إلى مصفوفة مربعة'],
  [/^the inverse needs a square matrix$/, 'يحتاج المعكوس إلى مصفوفة مربعة'],
  [/^the matrix is not rectangular$/, 'المصفوفة ليست مستطيلة'],
  [/^a matrix needs at least one row and one column$/, 'تحتاج المصفوفة إلى صف وعمود واحد على الأقل'],
  [/^the number of rows does not match the input$/, 'عدد الصفوف لا يطابق المدخلات'],
  [/^the matrices must have the same number of rows$/, 'يجب أن يكون للمصفوفتين عدد الصفوف نفسه'],
  [/^provide a matrix$/, 'أدخل مصفوفة'],
  [/^provide a right-hand side$/, 'أدخل الطرف الأيمن'],
  [/^choose an arithmetic operation$/, 'اختر عملية حسابية'],
  [/^choose an operation$/, 'اختر عملية'],
  [/^choose an integer n$/, 'اختر عددًا صحيحًا n'],
  [/^choose arithmetic, polar, power or roots$/, 'اختر حسابًا أو قطبية أو قوة أو جذورًا'],
  [/^choose whether to maximise or minimise$/, 'اختر تعظيمًا أو تصغيرًا'],
  [/^enter a number$/, 'أدخل عددًا'],
  [/^numbers must be finite$/, 'يجب أن تكون الأعداد منتهية'],
  [/^numbers must be numeric$/, 'يجب أن تكون الأعداد رقمية'],
  [/^coefficients must be finite$/, 'يجب أن تكون المعاملات منتهية'],
  [/^coefficients must be numbers$/, 'يجب أن تكون المعاملات أعدادًا'],
  [/^entries must be finite$/, 'يجب أن تكون العناصر منتهية'],
  [/^entries must be numbers$/, 'يجب أن تكون العناصر أعدادًا'],
  [/^a constraint does not match the number of variables$/, 'أحد القيود لا يطابق عدد المتغيرات'],
  [/^objective length does not match the number of variables$/, 'طول دالة الهدف لا يطابق عدد المتغيرات'],
  [/^the objective needs one coefficient per variable$/, 'تحتاج دالة الهدف إلى معامل واحد لكل متغير'],
  [/^the right-hand side needs one entry per row$/, 'يحتاج الطرف الأيمن إلى عنصر واحد لكل صف'],
  [/^missing variables$/, 'المتغيرات ناقصة'],
  [/^missing objective coefficients$/, 'معاملات دالة الهدف ناقصة'],
  [/^expected a unique solution$/, 'كان يُتوقَّع حل وحيد'],
  [/^expected infinitely many solutions$/, 'كان يُتوقَّع عدد لا نهائي من الحلول'],
  [/^invalid variable name '(.+)': use letters and digits, starting with a letter$/, 'اسم متغير غير صالح «$1»: استخدم حروفًا وأرقامًا يبدأها حرف'],
  [/^duplicate variable name '(.+)'$/, 'اسم متغير مكرر «$1»'],
  [/^iteration limit reached; the problem may be cycling$/, 'بُلِغ حد التكرار؛ قد تدور المسألة'],
  [/^(.+) needs a real and an imaginary part$/, '$1 يحتاج إلى جزء حقيقي وجزء تخيلي'],
  [/^row (\d+) has the wrong number of entries$/, 'الصف $1 يحتوي عددًا خاطئًا من العناصر'],
  [/^entry \((\d+), (\d+)\): /, 'العنصر ($1، $2): '],
  [/^constraint (\d+) /, 'القيد $1 '],
  [/^invalid input: /, 'مدخلات غير صالحة: '],
  [/^internal error.*$/, 'خطأ داخلي في المحرك']
];

export function localizeError(message: string): string {
  let value = message;
  for (const [pattern, replacement] of errors) {
    if (pattern.test(value)) {
      value = value.replace(pattern, replacement);
      break;
    }
  }
  // Nested parser errors such as "constraint 1 coefficient 2: enter a number".
  value = value
    .replace(/enter a number/g, 'أدخل عددًا')
    .replace(/numbers must be numeric/g, 'يجب أن تكون الأعداد رقمية')
    .replace(/numbers must be finite/g, 'يجب أن تكون الأعداد منتهية')
    .replace(/coefficients must be numbers/g, 'يجب أن تكون المعاملات أعدادًا')
    .replace(/coefficients must be finite/g, 'يجب أن تكون المعاملات منتهية');
  return arNumeric(value);
}
